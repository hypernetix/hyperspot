//! GTS Documentation Validator (DE0903)
//!
//! This tool validates GTS (Global Type System) identifiers in documentation files
//! (.md, .json, .yaml), complementing the Rust-based DE0901 lint that validates
//! GTS identifiers in source code.
//!
//! # Usage
//!
//! ```bash
//! # Validate docs with vendor filter
//! gts-docs-validator --vendor x docs modules libs examples
//!
//! # With exclusions
//! gts-docs-validator --vendor x --exclude "target/*" --exclude "docs/api/*" .
//!
//! # JSON output
//! gts-docs-validator --vendor x --json docs
//! ```

// CLI tools are expected to print to stdout/stderr
#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::exit,
    clippy::expect_used
)]

mod scanner;
mod validator;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use colored::Colorize;

use crate::scanner::{find_files, scan_file};
use crate::validator::ValidationResult;

/// GTS Documentation Validator (DE0903)
///
/// Validates GTS identifiers in .md, .json, and .yaml files.
/// Ensures all GTS IDs follow the correct format and optionally validates
/// that they use a specific vendor.
#[derive(Parser, Debug)]
#[command(name = "gts-docs-validator")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Paths to scan (files or directories)
    /// Defaults to: docs, modules, libs, examples
    #[arg(value_name = "PATH")]
    paths: Vec<PathBuf>,

    /// Expected vendor for all GTS IDs (validates vendor matches)
    /// Example: --vendor x ensures all IDs use vendor "x"
    ///
    /// Note: Example vendors are always tolerated: acme, globex, example, demo, test, sample, tutorial
    #[arg(long)]
    vendor: Option<String>,

    /// Exclude patterns (can be specified multiple times)
    /// Supports glob patterns. Example: --exclude "target/*" --exclude "docs/api/*"
    #[arg(long, short = 'e', action = clap::ArgAction::Append)]
    exclude: Vec<String>,

    /// Output results as JSON
    #[arg(long)]
    json: bool,

    /// Show verbose output including file scanning progress
    #[arg(long, short = 'v')]
    verbose: bool,
}

/// Default directories to scan if none specified
const DEFAULT_SCAN_DIRS: &[&str] = &["docs", "modules", "libs", "examples"];

fn main() -> ExitCode {
    let cli = Cli::parse();

    // Determine paths to scan
    let paths: Vec<PathBuf> = if cli.paths.is_empty() {
        DEFAULT_SCAN_DIRS
            .iter()
            .map(PathBuf::from)
            .filter(|p| p.exists())
            .collect()
    } else {
        // Validate explicitly provided paths exist
        for path in &cli.paths {
            if !path.exists() {
                eprintln!("Error: Path does not exist: {}", path.display());
                return ExitCode::FAILURE;
            }
        }
        cli.paths.clone()
    };

    if paths.is_empty() {
        eprintln!("No existing paths to scan. Provide paths explicitly.");
        return ExitCode::FAILURE;
    }

    if cli.verbose {
        let path_list: Vec<_> = paths.iter().map(|p| p.display().to_string()).collect();
        eprintln!("Scanning paths: {}", path_list.join(", "));
        if let Some(ref vendor) = cli.vendor {
            eprintln!("Expected vendor: {vendor}");
        }
    }

    // Find all files to scan
    let files = find_files(&paths, &cli.exclude, cli.verbose);

    if cli.verbose {
        eprintln!("Found {} files to scan", files.len());
    }

    if files.is_empty() {
        eprintln!("No files found to scan. Check paths and exclusion patterns.");
        return ExitCode::FAILURE;
    }

    // Scan all files and collect errors
    let mut result = ValidationResult::new(files.len());

    for file_path in &files {
        if cli.verbose {
            eprintln!("  Scanning: {}", file_path.display());
        }

        let file_errors = scan_file(file_path, cli.vendor.as_deref(), cli.verbose);
        result.add_errors(file_errors);
    }

    // Output results
    if cli.json {
        print_json_results(&result);
    } else {
        print_results(&result, cli.verbose);
    }

    if result.is_ok() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn print_json_results(result: &ValidationResult) {
    let output = serde_json::json!({
        "files_scanned": result.files_scanned,
        "errors_count": result.errors.len(),
        "ok": result.is_ok(),
        "errors": result.errors.iter().map(|e| {
            serde_json::json!({
                "file": e.file.display().to_string(),
                "line": e.line,
                "column": e.column,
                "gts_id": e.gts_id,
                "error": e.error,
            })
        }).collect::<Vec<_>>(),
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&output).expect("Failed to serialize JSON")
    );
}

fn print_results(result: &ValidationResult, verbose: bool) {
    println!();
    println!("{}", "=".repeat(80));
    println!("  {}", "GTS DOCUMENTATION VALIDATOR (DE0903)".bold());
    println!("{}", "=".repeat(80));
    println!();
    println!("  Files scanned: {}", result.files_scanned);
    println!("  Errors found:  {}", result.errors.len());
    println!();

    if !result.errors.is_empty() {
        println!("{}", "-".repeat(80));
        println!("  {}", "ERRORS".red().bold());
        println!("{}", "-".repeat(80));

        // Group errors by file
        let mut errors_by_file: std::collections::HashMap<&PathBuf, Vec<_>> =
            std::collections::HashMap::new();
        for err in &result.errors {
            errors_by_file.entry(&err.file).or_default().push(err);
        }

        let mut sorted_files: Vec<_> = errors_by_file.keys().collect();
        sorted_files.sort();

        for file_path in sorted_files {
            let file_errors = &errors_by_file[file_path];
            println!("\n  {}:", file_path.display().to_string().yellow());

            let mut sorted_errors: Vec<_> = file_errors.iter().collect();
            sorted_errors.sort_by_key(|e| e.line);

            for err in sorted_errors {
                println!(
                    "    Line {}:{} - {}",
                    err.line,
                    err.column,
                    err.gts_id.cyan()
                );
                println!("      Error: {}", err.error.red());
                if verbose && !err.context.is_empty() {
                    println!("      Context: {}", err.context.dimmed());
                }
            }
        }
        println!();
    }

    println!("{}", "=".repeat(80));
    if result.is_ok() {
        println!(
            "  STATUS: {} {}",
            "ALL GTS IDENTIFIERS VALID".green().bold(),
            "\u{2713}".green()
        );
    } else {
        println!(
            "  STATUS: {} {}",
            format!("{} INVALID GTS IDENTIFIERS FOUND", result.errors.len())
                .red()
                .bold(),
            "\u{2717}".red()
        );
        println!();
        println!("  To fix:");
        println!("    - Schema IDs must end with ~ (e.g., gts.x.core.type.v1~)");
        println!("    - Each segment needs 5 parts: vendor.org.package.type.version");
        println!("    - No hyphens allowed, use underscores");
        println!("    - Wildcards (*) only in filter/pattern contexts");
        if result
            .errors
            .iter()
            .any(|e| e.error.contains("Vendor mismatch"))
        {
            println!("    - Ensure all GTS IDs use the expected vendor");
        }
    }
    println!("{}", "=".repeat(80));
}
