use crate::config::{LoggingConfig, Section};
use std::io::IsTerminal;
use std::{
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use tracing::Level;
use tracing_subscriber::{filter::FilterFn, fmt};

use file_rotate::{
    compression::Compression,
    suffix::{AppendTimestamp, FileLimit},
    ContentLimit, FileRotate,
};

// -------- level helpers --------
fn parse_tracing_level(s: &str) -> Option<tracing::Level> {
    match s.to_ascii_lowercase().as_str() {
        "trace" => Some(Level::TRACE),
        "debug" => Some(Level::DEBUG),
        "info" => Some(Level::INFO),
        "warn" => Some(Level::WARN),
        "error" => Some(Level::ERROR),
        "off" | "none" => None,
        _ => Some(Level::INFO),
    }
}

// -------- filtering functions --------

type CrateFilter = FilterFn<Box<dyn Fn(&tracing::Metadata<'_>) -> bool + Send + Sync + 'static>>;

fn create_default_filter_for_crates(
    crate_names: &[String],
    max_level: tracing::Level,
) -> CrateFilter {
    let crates = crate_names.to_vec();
    FilterFn::new(Box::new(move |meta: &tracing::Metadata<'_>| {
        let t = meta.target();
        // If the target belongs to any of the listed crates, it's not default
        for c in &crates {
            if matches_crate_prefix(t, c) {
                return false;
            }
        }
        // Otherwise: everything else
        meta.level() <= &max_level
    }))
}

/// Returns true if target == crate_name or target starts with "crate_name::"
fn matches_crate_prefix(target: &str, crate_name: &str) -> bool {
    target == crate_name
        || (target.starts_with(crate_name) && target[crate_name.len()..].starts_with("::"))
}

// -------- rotating writer for files --------
#[derive(Clone)]
struct RotWriter(Arc<Mutex<FileRotate<AppendTimestamp>>>);

impl<'a> fmt::MakeWriter<'a> for RotWriter {
    type Writer = RotWriterHandle;
    fn make_writer(&'a self) -> Self::Writer {
        RotWriterHandle(self.0.clone())
    }
}

#[derive(Clone)]
struct RotWriterHandle(Arc<Mutex<FileRotate<AppendTimestamp>>>);

impl Write for RotWriterHandle {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.lock().unwrap().flush()
    }
}

use std::collections::HashMap;

// A writer handle that may be None (drops writes)
#[derive(Clone)]
struct RoutedWriterHandle(Option<RotWriterHandle>);

impl std::io::Write for RoutedWriterHandle {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if let Some(w) = &mut self.0 {
            w.write(buf)
        } else {
            // Drop silently; pretend we wrote everything
            Ok(buf.len())
        }
    }
    fn flush(&mut self) -> std::io::Result<()> {
        if let Some(w) = &mut self.0 {
            w.flush()
        } else {
            Ok(())
        }
    }
}

/// Route log records to different files by target prefix:
/// keys are *full* prefixes like "hyperspot::api_ingress"
struct MultiFileRouter {
    default: Option<RotWriter>, // default file (from "default" section), optional
    by_prefix: HashMap<String, RotWriter>, // subsystem → writer
}

impl MultiFileRouter {
    fn resolve_for(&self, target: &str) -> Option<RotWriterHandle> {
        for (crate_name, wr) in &self.by_prefix {
            if matches_crate_prefix(target, crate_name) {
                return Some(RotWriterHandle(wr.0.clone()));
            }
        }
        // Fallback to default file
        self.default.as_ref().map(|w| RotWriterHandle(w.0.clone()))
    }

    fn is_empty(&self) -> bool {
        self.default.is_none() && self.by_prefix.is_empty()
    }
}

impl<'a> fmt::MakeWriter<'a> for MultiFileRouter {
    type Writer = RoutedWriterHandle;

    fn make_writer(&'a self) -> Self::Writer {
        // Used rarely; use default file if any
        RoutedWriterHandle(self.default.as_ref().map(|w| RotWriterHandle(w.0.clone())))
    }

    fn make_writer_for(&'a self, meta: &tracing::Metadata<'_>) -> Self::Writer {
        let target = meta.target();
        RoutedWriterHandle(self.resolve_for(target))
    }
}

// -------- config extraction (fix clippy lifetime warning) --------

struct ConfigData<'a> {
    default_section: Option<&'a Section>,
    crate_sections: Vec<(String, &'a Section)>,
    crate_names: Vec<String>,
}

fn extract_config_data(cfg: &LoggingConfig) -> ConfigData<'_> {
    // Avoid explicit lifetime annotations here; let the compiler infer them.
    let crate_sections = cfg
        .iter()
        .filter(|(k, _)| k.as_str() != "default")
        .map(|(k, v)| (k.clone(), v))
        .collect::<Vec<_>>();

    let crate_names = crate_sections.iter().map(|(n, _)| n.clone()).collect();

    ConfigData {
        default_section: cfg.get("default"),
        crate_sections,
        crate_names,
    }
}

// -------- path resolution helpers --------

/// Resolve a log file path against `base_dir` (home_dir).
/// Absolute paths are kept as-is; relative paths are joined with `base_dir`.
fn resolve_log_path(file: &str, base_dir: &Path) -> PathBuf {
    let p = Path::new(file);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        base_dir.join(p)
    }
}

/// Create a rotating writer for log files, ensuring the parent directory exists.
/// `log_path` must be an absolute or already-resolved path.
fn create_rotating_writer_at_path(
    log_path: &Path,
    max_bytes: usize,
) -> Result<RotWriter, Box<dyn std::error::Error + Send + Sync>> {
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let rot = FileRotate::new(
        log_path,
        AppendTimestamp::default(FileLimit::Age(chrono::Duration::days(1))),
        ContentLimit::BytesSurpassed(max_bytes),
        Compression::None,
        None, // file permissions (Unix only)
    );

    Ok(RotWriter(Arc::new(Mutex::new(rot))))
}

// -------- public init --------

/// Initialize logging from a configuration.
/// - `cfg`: LoggingConfig containing the logging sections
/// - `base_dir`: base directory used to resolve relative log file paths (usually server.home_dir)
pub fn init_logging_from_config(cfg: &LoggingConfig, base_dir: &Path) {
    // Bridge `log` → `tracing` *before* installing the subscriber
    let _ = tracing_log::LogTracer::init();

    if cfg.is_empty() {
        init_default_logging();
        return;
    }

    let config_data = extract_config_data(cfg);
    let console_targets = build_console_targets(&config_data);
    let file_router = build_file_router(&config_data, base_dir);
    let file_targets = build_file_targets(&config_data, file_router.default.is_some());

    build_logging_layers(config_data, console_targets, file_targets, file_router);
}

fn init_default_logging() {
    use tracing_subscriber::fmt;
    let _ = fmt()
        .with_target(true)
        .with_timer(fmt::time::UtcTime::rfc_3339())
        .try_init();
}

fn build_console_targets(config: &ConfigData) -> tracing_subscriber::filter::Targets {
    use tracing::level_filters::LevelFilter;
    use tracing_subscriber::filter::Targets;

    let mut targets = Targets::new().with_default(LevelFilter::OFF);

    // Add explicit crate targets
    for (crate_name, section) in &config.crate_sections {
        if let Some(level) =
            parse_tracing_level(&section.console_level).map(LevelFilter::from_level)
        {
            targets = targets.with_target(crate_name.clone(), level);
        }
    }

    targets
}

fn build_file_router(config: &ConfigData, base_dir: &Path) -> MultiFileRouter {
    let mut router = MultiFileRouter {
        default: None,
        by_prefix: HashMap::new(),
    };

    // Setup default file writer
    if let Some(section) = config.default_section {
        router.default = create_default_file_writer(section, base_dir);
    }

    // Setup per-crate file writers
    for (crate_name, section) in &config.crate_sections {
        if let Some(writer) = create_crate_file_writer(crate_name, section, base_dir) {
            router.by_prefix.insert(crate_name.clone(), writer);
        }
    }

    router
}

fn create_default_file_writer(section: &Section, base_dir: &Path) -> Option<RotWriter> {
    if section.file.trim().is_empty() {
        return None;
    }

    let max_bytes = section.max_size_mb.unwrap_or(100) * 1024 * 1024;
    let log_path = resolve_log_path(&section.file, base_dir);

    match create_rotating_writer_at_path(&log_path, max_bytes as usize) {
        Ok(writer) => Some(writer),
        Err(_) => {
            eprintln!(
                "Failed to initialize default log file '{}'",
                log_path.to_string_lossy()
            );
            None
        }
    }
}

fn create_crate_file_writer(
    crate_name: &str,
    section: &Section,
    base_dir: &Path,
) -> Option<RotWriter> {
    if section.file.trim().is_empty() {
        return None;
    }

    let max_bytes = section.max_size_mb.unwrap_or(100) * 1024 * 1024;
    let log_path = resolve_log_path(&section.file, base_dir);

    match create_rotating_writer_at_path(&log_path, max_bytes as usize) {
        Ok(writer) => Some(writer),
        Err(e) => {
            eprintln!(
                "Failed to init log file for subsystem '{}': {} ({})",
                crate_name,
                log_path.to_string_lossy(),
                e
            );
            None
        }
    }
}

fn build_file_targets(
    config: &ConfigData,
    _has_default_file: bool,
) -> tracing_subscriber::filter::Targets {
    use tracing::level_filters::LevelFilter;
    use tracing_subscriber::filter::Targets;

    let mut targets = Targets::new().with_default(LevelFilter::OFF);

    for (crate_name, section) in &config.crate_sections {
        if section.file.trim().is_empty() {
            continue;
        }

        if let Some(level) = parse_tracing_level(&section.file_level).map(LevelFilter::from_level) {
            targets = targets.with_target(crate_name.clone(), level);
        }
    }

    targets
}

fn build_logging_layers(
    config: ConfigData,
    console_targets: tracing_subscriber::filter::Targets,
    file_targets: tracing_subscriber::filter::Targets,
    file_router: MultiFileRouter,
) {
    use tracing_subscriber::{fmt, layer::SubscriberExt, prelude::*, Registry};

    let ansi = std::io::stdout().is_terminal();

    let console_layer = fmt::layer()
        .with_ansi(ansi)
        .with_target(true)
        .with_level(true)
        .with_timer(fmt::time::UtcTime::rfc_3339())
        .with_filter(console_targets);

    if file_router.is_empty() {
        let _ = Registry::default().with(console_layer).try_init();
        return;
    }

    let router_for_explicit = MultiFileRouter {
        default: file_router.default.clone(),
        by_prefix: file_router.by_prefix.clone(),
    };

    let explicit_file_layer = fmt::layer()
        .json()
        .with_ansi(false)
        .with_target(true)
        .with_level(true)
        .with_timer(fmt::time::UtcTime::rfc_3339())
        .with_writer(router_for_explicit)
        .with_filter(file_targets);

    // Add default layers if configured
    if let Some(default_section) = config.default_section {
        // Console default layer
        if let Some(console_level) = parse_tracing_level(&default_section.console_level) {
            let console_default = fmt::layer()
                .with_ansi(ansi)
                .with_target(true)
                .with_level(true)
                .with_timer(fmt::time::UtcTime::rfc_3339())
                .with_filter(create_default_filter_for_crates(
                    &config.crate_names,
                    console_level,
                ));

            // File default layer (if file is configured)
            if file_router.default.is_some() {
                if let Some(file_level) = parse_tracing_level(&default_section.file_level) {
                    let file_default = fmt::layer()
                        .json()
                        .with_ansi(false)
                        .with_target(true)
                        .with_level(true)
                        .with_timer(fmt::time::UtcTime::rfc_3339())
                        .with_writer(file_router)
                        .with_filter(create_default_filter_for_crates(
                            &config.crate_names,
                            file_level,
                        ));

                    let _ = Registry::default()
                        .with(console_layer)
                        .with(explicit_file_layer)
                        .with(console_default)
                        .with(file_default)
                        .try_init();
                    return;
                }
            }

            let _ = Registry::default()
                .with(console_layer)
                .with(explicit_file_layer)
                .with(console_default)
                .try_init();
            return;
        }
    }

    let _ = Registry::default()
        .with(console_layer)
        .with(explicit_file_layer)
        .try_init();
}

// =================== tests ===================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{default_logging_config, AppConfig};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_logging_level_parsing() {
        assert_eq!(parse_tracing_level("trace"), Some(Level::TRACE));
        assert_eq!(parse_tracing_level("DEBUG"), Some(Level::DEBUG));
        assert_eq!(parse_tracing_level("Info"), Some(Level::INFO));
        assert_eq!(parse_tracing_level("warn"), Some(Level::WARN));
        assert_eq!(parse_tracing_level("ERROR"), Some(Level::ERROR));
        assert_eq!(parse_tracing_level("off"), None);
        assert_eq!(parse_tracing_level("none"), None);
        assert_eq!(parse_tracing_level("invalid"), Some(Level::INFO)); // defaults to INFO
    }

    #[test]
    fn test_extract_config_data_lifetimes() {
        let mut cfg = default_logging_config();
        // add one crate section
        cfg.insert(
            "my_crate".into(),
            Section {
                console_level: "info".into(),
                file: "logs/my_crate.log".into(),
                file_level: "debug".into(),
                max_age_days: Some(7),
                max_backups: Some(3),
                max_size_mb: Some(10),
            },
        );

        let data = super::extract_config_data(&cfg);
        assert!(data.default_section.is_some());
        assert_eq!(data.crate_sections.len(), 1);
        assert_eq!(data.crate_names, vec!["my_crate".to_string()]);
    }

    #[test]
    fn test_file_paths_resolved_against_home_dir() {
        // set up a fake home_dir
        let tmp = tempdir().unwrap();
        let base_dir = tmp.path();

        let section = Section {
            console_level: "info".into(),
            file: "logs/test.log".into(), // relative path
            file_level: "debug".into(),
            max_age_days: Some(7),
            max_backups: Some(2),
            max_size_mb: Some(1),
        };

        let resolved = super::resolve_log_path(&section.file, base_dir);
        assert!(resolved.starts_with(base_dir));
        assert!(resolved.ends_with("logs/test.log"));
    }

    #[test]
    fn test_create_rotating_writer_at_path_creates_parent() {
        let tmp = tempdir().unwrap();
        let p = tmp.path().join("nested/dir/app.log");

        let res = super::create_rotating_writer_at_path(&p, 128 * 1024);
        assert!(res.is_ok(), "writer should be created");
        assert!(p.parent().unwrap().exists(), "parent dir must be created");
    }

    #[test]
    fn test_config_logging_integration_with_base_dir() {
        // prepare a config on disk
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.yaml");

        let yaml_content = r#"
server:
  home_dir: "~/.test_hyperspot"
  host: "127.0.0.1"
  port: 8088

database:
  servers:
    test_sqlite:
      dsn: "sqlite://test.db"

logging:
  default:
    console_level: info
    file: ""
    file_level: debug
  api_ingress:
    console_level: debug
    file: "logs/api_test.log"
    file_level: warn
    max_size_mb: 5
    max_backups: 2

modules:
  api_ingress:
    config:
      bind_addr: "127.0.0.1:8088"
"#;

        fs::write(&config_path, yaml_content).unwrap();

        // Load config (home_dir is normalized inside)
        let config = AppConfig::load_layered(&config_path).unwrap();

        // Build writer path using our resolver to ensure it points under home_dir
        let log_rel = "logs/api_test.log";
        let abs = super::resolve_log_path(log_rel, Path::new(&config.server.home_dir));
        assert!(abs.starts_with(&config.server.home_dir));
        assert!(abs.ends_with("logs/api_test.log"));
    }
}
