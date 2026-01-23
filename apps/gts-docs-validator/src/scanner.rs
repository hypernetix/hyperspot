//! File scanning functionality for GTS documentation validation

use std::fs;
use std::path::{Path, PathBuf};

use glob::Pattern;
use regex::Regex;
use walkdir::WalkDir;

use crate::validator::{GtsError, is_bad_example_context, is_wildcard_context, validate_gts_id};

/// File patterns to scan
const FILE_PATTERNS: &[&str] = &["*.md", "*.json", "*.yaml", "*.yml"];

/// Directories to skip
const SKIP_DIRS: &[&str] = &["target", "node_modules", ".git", "vendor", ".gts-spec"];

/// Files to skip (relative paths)
const SKIP_FILES: &[&str] = &["docs/api/api.json"];

/// Patterns that look like GTS but aren't (false positives)
const FALSE_POSITIVE_PATTERNS: &[&str] = &[
    r"^gts\.rs$",     // Rust file named gts.rs
    r"^gts\.[a-z]+$", // Single component like gts.rs, gts.py
];

/// Pattern to find GTS-looking strings
/// Must have at least 2 dots after gts. to catch potential GTS IDs
/// Include hyphen so we can catch and report invalid hyphens
fn gts_pattern() -> Regex {
    Regex::new(r"gts\.[a-z0-9_.*~-]+\.[a-z0-9_.*~-]+").expect("Invalid regex pattern")
}

/// Check if a path matches any of the exclude patterns
fn matches_exclude(path: &Path, exclude_patterns: &[Pattern]) -> bool {
    let path_str = path.to_string_lossy();
    for pattern in exclude_patterns {
        if pattern.matches(&path_str)
            || path
                .file_name()
                .is_some_and(|name| pattern.matches(&name.to_string_lossy()))
        {
            return true;
        }
        // Also try matching just the file/dir name
        if let Some(name) = path.file_name()
            && pattern.matches(&name.to_string_lossy())
        {
            return true;
        }
    }
    false
}

/// Check if path contains any skip directories
fn in_skip_dir(path: &Path) -> bool {
    for component in path.components() {
        if let std::path::Component::Normal(name) = component
            && SKIP_DIRS.iter().any(|skip| name.to_string_lossy() == *skip)
        {
            return true;
        }
    }
    false
}

/// Check if file matches any of the file patterns
fn matches_file_pattern(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        let with_dot = format!("*.{ext_str}");
        FILE_PATTERNS.iter().any(|p| *p == with_dot)
    } else {
        false
    }
}

/// Find all files to scan in the given paths
#[must_use]
pub fn find_files(paths: &[PathBuf], exclude: &[String], verbose: bool) -> Vec<PathBuf> {
    let mut files = Vec::new();

    // Parse exclude patterns
    let exclude_patterns: Vec<Pattern> = exclude
        .iter()
        .filter_map(|p| match Pattern::new(p) {
            Ok(pat) => Some(pat),
            Err(e) => {
                if verbose {
                    eprintln!("Warning: Invalid exclude pattern '{p}': {e}");
                }
                None
            }
        })
        .collect();

    for path in paths {
        if path.is_file() {
            if matches_file_pattern(path) && !matches_exclude(path, &exclude_patterns) {
                files.push(path.clone());
            }
        } else if path.is_dir() {
            for entry in WalkDir::new(path)
                .follow_links(true)
                .into_iter()
                .filter_map(Result::ok)
            {
                let file_path = entry.path();

                // Skip directories in skip list
                if in_skip_dir(file_path) {
                    continue;
                }

                // Only process files
                if !file_path.is_file() {
                    continue;
                }

                // Check file pattern
                if !matches_file_pattern(file_path) {
                    continue;
                }

                // Check exclude patterns
                if matches_exclude(file_path, &exclude_patterns) {
                    if verbose {
                        eprintln!("  Excluding: {}", file_path.display());
                    }
                    continue;
                }

                // Check against skip files
                let rel_path = file_path.to_string_lossy();
                if SKIP_FILES.iter().any(|skip| rel_path.contains(skip)) {
                    if verbose {
                        eprintln!("  Skipping: {}", file_path.display());
                    }
                    continue;
                }

                files.push(file_path.to_path_buf());
            }
        }
    }

    files.sort();
    files.dedup();
    files
}

/// Check if a matched string is a false positive
fn is_false_positive(gts_id: &str) -> bool {
    for pattern in FALSE_POSITIVE_PATTERNS {
        if let Ok(re) = Regex::new(pattern)
            && re.is_match(gts_id)
        {
            return true;
        }
    }
    false
}

/// Scan a single file for GTS identifiers and validate them
#[must_use]
pub fn scan_file(path: &Path, expected_vendor: Option<&str>, verbose: bool) -> Vec<GtsError> {
    let mut errors = Vec::new();

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            if verbose {
                eprintln!("  Warning: Could not read {}: {e}", path.display());
            }
            return vec![GtsError {
                file: path.to_path_buf(),
                line: 0,
                column: 0,
                gts_id: String::new(),
                error: format!("Failed to read file: {e}"),
                context: String::new(),
            }];
        }
    };

    let lines: Vec<&str> = content.lines().collect();
    let gts_re = gts_pattern();

    for (line_idx, line) in lines.iter().enumerate() {
        let line_num = line_idx + 1;

        for mat in gts_re.find_iter(line) {
            let gts_id = mat.as_str();
            let col = mat.start() + 1;

            // Skip false positives
            if is_false_positive(gts_id) {
                continue;
            }

            // Skip if in bad example context
            let prev_lines: Vec<&str> = lines[..line_idx].to_vec();
            if is_bad_example_context(line, &prev_lines) {
                continue;
            }

            // Check if wildcards are allowed
            let allow_wildcards = is_wildcard_context(line, mat.start());

            // Validate the identifier
            let validation_errors = validate_gts_id(gts_id, expected_vendor, allow_wildcards);

            for err in validation_errors {
                // Extract surrounding text for context (safely handle UTF-8 boundaries)
                let ctx_start = mat.start().saturating_sub(20);
                let ctx_end = (mat.end() + 20).min(line.len());

                // Find valid UTF-8 boundaries
                // safe_start: nearest boundary at or before ctx_start
                let safe_start = line
                    .char_indices()
                    .map(|(i, _)| i)
                    .take_while(|&i| i <= ctx_start)
                    .last()
                    .unwrap_or(0);
                // safe_end: nearest boundary at or after ctx_end
                let safe_end = line
                    .char_indices()
                    .map(|(i, c)| i + c.len_utf8())
                    .find(|&i| i >= ctx_end)
                    .unwrap_or(line.len());

                let mut ctx_text = line[safe_start..safe_end].to_owned();
                if safe_start > 0 {
                    ctx_text = format!("...{ctx_text}");
                }
                if safe_end < line.len() {
                    ctx_text = format!("{ctx_text}...");
                }

                errors.push(GtsError {
                    file: path.to_path_buf(),
                    line: line_num,
                    column: col,
                    gts_id: gts_id.to_owned(),
                    error: err,
                    context: ctx_text,
                });
            }
        }
    }

    errors
}

#[cfg(test)]
#[allow(clippy::non_ascii_literal)] // Allow non-ASCII literals - we're testing UTF-8 handling
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_matches_file_pattern() {
        assert!(matches_file_pattern(Path::new("docs/README.md")));
        assert!(matches_file_pattern(Path::new("config.json")));
        assert!(matches_file_pattern(Path::new("schema.yaml")));
        assert!(!matches_file_pattern(Path::new("main.rs")));
        assert!(!matches_file_pattern(Path::new("script.py")));
    }

    #[test]
    fn test_in_skip_dir() {
        assert!(in_skip_dir(Path::new("project/target/debug/file.md")));
        assert!(in_skip_dir(Path::new("node_modules/package/README.md")));
        assert!(!in_skip_dir(Path::new("docs/README.md")));
    }

    #[test]
    fn test_scan_file_valid() {
        let mut file = NamedTempFile::with_suffix(".md").unwrap();
        writeln!(
            file,
            "# Documentation\n\nThe type is `gts.x.core.modkit.plugin.v1~`"
        )
        .unwrap();

        let errors = scan_file(file.path(), None, false);
        assert!(errors.is_empty(), "Unexpected errors: {errors:?}");
    }

    #[test]
    fn test_scan_file_invalid_segment() {
        let mut file = NamedTempFile::with_suffix(".md").unwrap();
        writeln!(
            file,
            "# Documentation\n\nThe type is: `gts.x.core.plugin.v1~`"
        )
        .unwrap();

        let errors = scan_file(file.path(), None, false);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].error.contains("5 components"));
    }

    #[test]
    fn test_scan_file_vendor_mismatch() {
        let mut file = NamedTempFile::with_suffix(".md").unwrap();
        writeln!(
            file,
            "# Documentation\n\nThe type is: `gts.hx.core.modkit.plugin.v1~`"
        )
        .unwrap();

        let errors = scan_file(file.path(), Some("x"), false);
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.error.contains("Vendor mismatch")));
    }

    #[test]
    fn test_scan_file_bad_example_skipped() {
        let mut file = NamedTempFile::with_suffix(".md").unwrap();
        writeln!(
            file,
            "# Documentation\n\n## Example: Bad\n\nInvalid format: `gts.x.core.v1~`"
        )
        .unwrap();

        let errors = scan_file(file.path(), None, false);
        assert!(errors.is_empty(), "Bad example should be skipped");
    }

    #[test]
    fn test_scan_file_wildcard_in_filter() {
        let mut file = NamedTempFile::with_suffix(".md").unwrap();
        writeln!(file, "Use `$filter=type_id eq 'gts.x.*'` to filter.").unwrap();

        let errors = scan_file(file.path(), None, false);
        assert!(
            errors.is_empty(),
            "Wildcards in filter context should be allowed"
        );
    }

    #[test]
    fn test_utf8_boundary_alignment_emoji() {
        // Test with emoji (4-byte UTF-8) before GTS ID
        // Using hyphen in segment makes it genuinely malformed
        let mut file = NamedTempFile::with_suffix(".md").unwrap();
        writeln!(file, "🚀 The type is `gts.x-vendor.org.pkg.type.v1~` here").unwrap();

        let errors = scan_file(file.path(), None, false);
        assert!(
            !errors.is_empty(),
            "Should detect malformed GTS ID with hyphen"
        );
        // Context should not panic and should include valid UTF-8
        assert!(!errors[0].context.is_empty());
        // Verify context is valid UTF-8 by checking it doesn't panic
        let _ = errors[0].context.chars().count();
    }

    #[test]
    fn test_utf8_boundary_alignment_multibyte_chars() {
        // Test with various multibyte characters (Chinese, Arabic, etc.)
        // Missing tilde at end makes it malformed
        let mut file = NamedTempFile::with_suffix(".md").unwrap();
        writeln!(
            file,
            "中文测试 العربية `gts.x.core.modkit.plugin.v1` тест ελληνικά"
        )
        .unwrap();

        let errors = scan_file(file.path(), None, false);
        assert!(!errors.is_empty(), "Should detect GTS ID missing tilde");
        // Context extraction should not panic with multibyte chars
        assert!(!errors[0].context.is_empty());
        // Verify the context is valid UTF-8
        let char_count = errors[0].context.chars().count();
        assert!(char_count > 0, "Context should contain characters");
    }

    #[test]
    fn test_utf8_boundary_alignment_at_start() {
        // Test GTS ID at the very start with multibyte chars after
        // Too few segments (only 4 instead of 5)
        let mut file = NamedTempFile::with_suffix(".md").unwrap();
        writeln!(file, "`gts.x.core.type.v1~` 日本語テスト").unwrap();

        let errors = scan_file(file.path(), None, false);
        assert!(!errors.is_empty(), "Should detect too few segments");
        // safe_start should be 0, safe_end should align properly
        let _ = errors[0].context.chars().count();
    }

    #[test]
    fn test_utf8_boundary_alignment_at_end() {
        // Test GTS ID near end of line with multibyte chars before
        // Hyphen makes it malformed
        let mut file = NamedTempFile::with_suffix(".md").unwrap();
        writeln!(file, "한글 테스트 🎉 `gts.my-vendor.org.pkg.type.v1~`").unwrap();

        let errors = scan_file(file.path(), None, false);
        assert!(!errors.is_empty(), "Should detect hyphen in GTS ID");
        // safe_end should align to line.len() properly
        let _ = errors[0].context.chars().count();
    }

    #[test]
    fn test_utf8_boundary_alignment_mixed_widths() {
        // Test with mix of 1-byte, 2-byte, 3-byte, and 4-byte UTF-8 chars
        // Missing version prefix 'v' makes it malformed
        let mut file = NamedTempFile::with_suffix(".md").unwrap();
        writeln!(
            file,
            "ASCII ñ 中 🎨 `gts.x.core.modkit.plugin.1~` 🚀 文 ü text"
        )
        .unwrap();

        let errors = scan_file(file.path(), None, false);
        assert!(!errors.is_empty(), "Should detect missing 'v' in version");
        let context = &errors[0].context;

        // Verify context doesn't truncate characters
        let _ = context.chars().count();

        // Context should be valid UTF-8 slice (no panic on string operations)
        assert!(!context.is_empty());
        assert!(context.is_char_boundary(0));
        assert!(context.is_char_boundary(context.len()));
    }

    #[test]
    fn test_utf8_context_window_stability() {
        // Verify that context windows are stable regardless of char position
        let mut file = NamedTempFile::with_suffix(".md").unwrap();
        // Create a line where the 20-char offset lands in middle of multibyte char
        let prefix = "🎨🎨🎨🎨🎨"; // 5 emoji = 20 bytes
        // Too few segments (only 4) makes it malformed
        writeln!(file, "{prefix}`gts.x.core.pkg.v1~` suffix").unwrap();

        let errors = scan_file(file.path(), None, false);
        assert!(!errors.is_empty(), "Should detect too few segments");

        // Context should start at a valid boundary (at or before offset 20)
        // and not panic or produce invalid UTF-8
        let context = &errors[0].context;
        assert!(context.is_char_boundary(0));
        assert!(context.is_char_boundary(context.len()));

        // Should contain the GTS ID
        assert!(context.contains("gts.x.core.pkg.v1~"));
    }
}
