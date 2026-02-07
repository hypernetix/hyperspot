use super::super::config::{LoggingConfig, Section};
use anyhow::Context;
use std::collections::HashMap;
use std::io;
use std::io::{IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tracing_subscriber::{Layer, fmt, util::SubscriberInitExt};

// ========== OTEL-agnostic layer type (compiles with/without the feature) ==========
#[cfg(feature = "otel")]
pub type OtelLayer = tracing_opentelemetry::OpenTelemetryLayer<
    tracing_subscriber::Registry,
    opentelemetry_sdk::trace::Tracer,
>;
#[cfg(not(feature = "otel"))]
pub type OtelLayer = ();

// Keep a guard for non-blocking console to avoid being dropped.
static CONSOLE_GUARD: std::sync::OnceLock<tracing_appender::non_blocking::WorkerGuard> =
    std::sync::OnceLock::new();

// ================= level helpers =================

/// Returns true if target == `crate_name` or target starts with "`crate_name::`"
fn matches_crate_prefix(target: &str, crate_name: &str) -> bool {
    target == crate_name
        || (target.starts_with(crate_name) && target[crate_name.len()..].starts_with("::"))
}

// ================= rotating writer for files =================

use file_rotate::{
    ContentLimit, FileRotate,
    compression::Compression,
    suffix::{AppendTimestamp, FileLimit},
};

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

impl RotWriterHandle {
    fn try_lock(
        &mut self,
    ) -> std::io::Result<std::sync::MutexGuard<'_, FileRotate<AppendTimestamp>>> {
        self.0
            .try_lock()
            .map_err(|e| io::Error::other(format!("Lock failed: {e}")))
    }
}

impl Write for RotWriterHandle {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.try_lock()?.write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.try_lock()?.flush()
    }
}

// A writer handle that may be None (drops writes)
#[derive(Clone)]
struct RoutedWriterHandle(Option<RotWriterHandle>);

impl Write for RoutedWriterHandle {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if let Some(w) = &mut self.0 {
            w.write(buf)
        } else {
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
/// keys are *full* prefixes like "`hyperspot::api_gateway`"
#[derive(Clone)]
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
        self.default.as_ref().map(|w| RotWriterHandle(w.0.clone()))
    }

    fn is_empty(&self) -> bool {
        self.default.is_none() && self.by_prefix.is_empty()
    }
}

impl<'a> fmt::MakeWriter<'a> for MultiFileRouter {
    type Writer = RoutedWriterHandle;

    fn make_writer(&'a self) -> Self::Writer {
        RoutedWriterHandle(self.default.as_ref().map(|w| RotWriterHandle(w.0.clone())))
    }

    fn make_writer_for(&'a self, meta: &tracing::Metadata<'_>) -> Self::Writer {
        let target = meta.target();
        RoutedWriterHandle(self.resolve_for(target))
    }
}

// ================= config extraction =================

struct ConfigData<'a> {
    default_section: Option<&'a Section>,
    crate_sections: Vec<(String, &'a Section)>,
}

fn extract_config_data(cfg: &LoggingConfig) -> ConfigData<'_> {
    let crate_sections = cfg
        .iter()
        .filter(|(k, _)| k.as_str() != "default")
        .map(|(k, v)| (k.clone(), v))
        .collect::<Vec<_>>();

    ConfigData {
        default_section: cfg.get("default"),
        crate_sections,
    }
}

// ================= path helpers =================

fn resolve_log_path(file: &str, base_dir: &Path) -> PathBuf {
    let p = Path::new(file);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        base_dir.join(p)
    }
}

fn create_rotating_writer_at_path(
    log_path: &Path,
    max_bytes: usize,
    max_age_days: Option<u32>,
    max_backups: Option<usize>,
) -> Result<RotWriter, Box<dyn std::error::Error + Send + Sync>> {
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Respect retention policy: prefer MaxFiles if provided, else Age
    let max_age_days = max_age_days.unwrap_or(1);
    let age = chrono::Duration::try_days(i64::from(max_age_days))
        .with_context(|| format!("Invalid max_age_days: {max_age_days}"))?;
    let limit = if let Some(n) = max_backups {
        FileLimit::MaxFiles(n)
    } else {
        FileLimit::Age(age)
    };

    let rot = FileRotate::new(
        log_path,
        AppendTimestamp::default(limit),
        ContentLimit::BytesSurpassed(max_bytes),
        Compression::None,
        None,
    );

    Ok(RotWriter(Arc::new(Mutex::new(rot))))
}

// ================= public init (drop-in API kept) =================

/// Unified initializer used by both functions above.
#[allow(unknown_lints, de1301_no_print_macros)] // runs before tracing subscriber is installed
pub fn init_logging_unified(cfg: &LoggingConfig, base_dir: &Path, otel_layer: Option<OtelLayer>) {
    // Bridge `log` → `tracing` *before* installing the subscriber
    if let Err(e) = tracing_log::LogTracer::init() {
        eprintln!("LogTracer init skipped: {e}");
    }

    let data = extract_config_data(cfg);

    if data.crate_sections.is_empty() && data.default_section.is_none() {
        // Minimal fallback (INFO to console; honors RUST_LOG)
        init_minimal(otel_layer);
        return;
    }

    // Build targets once, using a generic builder for different sinks
    let file_router = build_file_router(&data, base_dir);

    let console_targets = build_targets(&data, SinkKind::Console);
    let file_targets = build_targets(
        &data,
        SinkKind::File {
            has_default_file: file_router.default.is_some(),
        },
    );

    install_subscriber(&console_targets, &file_targets, file_router, otel_layer);
}

// ================= generic targets builder =================

use tracing::level_filters::LevelFilter;
use tracing_subscriber::filter::Targets;

/// Different "sinks" (destinations) for which we build Targets.
/// Only differences: which level field we read, whether the sink is active, and default fallback.
#[derive(Clone, Copy)]
enum SinkKind {
    Console,
    File { has_default_file: bool },
}

/// Noisy crates that should be filtered to WARN level to avoid debug spam
const NOISY_CRATES: &[&str] = &["h2"];

fn build_targets(config: &ConfigData, kind: SinkKind) -> Targets {
    match kind {
        SinkKind::Console => {
            // default level
            let default_level = config
                .default_section
                .and_then(|s| s.console_level)
                .map_or(LevelFilter::INFO, LevelFilter::from_level);

            // start with default
            let mut targets = Targets::new().with_default(default_level);

            // Suppress noisy low-level crates to WARN unless they need DEBUG/TRACE
            for crate_name in NOISY_CRATES {
                targets = targets.with_target(*crate_name, LevelFilter::WARN);
            }

            // per-crate rules (console sink is always "active")
            for (crate_name, section) in &config.crate_sections {
                if let Some(level) = section.console_level.map(LevelFilter::from_level) {
                    targets = targets.with_target(crate_name.clone(), level);
                }
            }

            targets
        }

        SinkKind::File { has_default_file } => {
            // default level depends on whether there is a default file sink
            let default_level = config.default_section.and_then(|s| s.file_level).map_or(
                if has_default_file {
                    LevelFilter::INFO
                } else {
                    LevelFilter::OFF
                },
                LevelFilter::from_level,
            );

            let mut targets = Targets::new().with_default(default_level);

            // per-crate rules: file sink is "active" only when path is present
            for (crate_name, section) in &config.crate_sections {
                if section.file.trim().is_empty() {
                    continue;
                }
                if let Some(level) = section.file_level.map(LevelFilter::from_level) {
                    targets = targets.with_target(crate_name.clone(), level);
                }
            }

            targets
        }
    }
}

// ================= building routers =================

fn build_file_router(config: &ConfigData, base_dir: &Path) -> MultiFileRouter {
    let mut router = MultiFileRouter {
        default: None,
        by_prefix: HashMap::new(),
    };

    if let Some(section) = config.default_section {
        router.default = create_default_file_writer(section, base_dir);
    }

    for (crate_name, section) in &config.crate_sections {
        if let Some(writer) = create_crate_file_writer(crate_name, section, base_dir) {
            router.by_prefix.insert(crate_name.clone(), writer);
        }
    }

    router
}

trait HasMaxSizeBytes {
    fn max_size_bytes(&self) -> usize;
}

const DEFAULT_SECTION_MAX_SIZE_MB: usize = 100;

impl HasMaxSizeBytes for Section {
    fn max_size_bytes(&self) -> usize {
        self.max_size_mb
            .map(|mb| mb * 1024 * 1024)
            .and_then(|b| usize::try_from(b).ok())
            .unwrap_or(DEFAULT_SECTION_MAX_SIZE_MB * 1024 * 1024)
    }
}

#[allow(unknown_lints, de1301_no_print_macros)] // runs during logging init, before tracing is available
fn create_default_file_writer(section: &Section, base_dir: &Path) -> Option<RotWriter> {
    if section.file.trim().is_empty() {
        return None;
    }

    let max_bytes = section.max_size_bytes();
    let log_path = resolve_log_path(&section.file, base_dir);

    if let Ok(writer) = create_rotating_writer_at_path(
        &log_path,
        max_bytes,
        section.max_age_days,
        section.max_backups,
    ) {
        Some(writer)
    } else {
        eprintln!(
            "Failed to initialize default log file '{}'",
            log_path.to_string_lossy()
        );
        None
    }
}

#[allow(unknown_lints, de1301_no_print_macros)] // runs during logging init, before tracing is available
fn create_crate_file_writer(
    crate_name: &str,
    section: &Section,
    base_dir: &Path,
) -> Option<RotWriter> {
    if section.file.trim().is_empty() {
        return None;
    }

    let max_bytes = section.max_size_bytes();
    let log_path = resolve_log_path(&section.file, base_dir);

    match create_rotating_writer_at_path(
        &log_path,
        max_bytes,
        section.max_age_days,
        section.max_backups,
    ) {
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

// ================= registry & layers =================

fn install_subscriber(
    console_targets: &tracing_subscriber::filter::Targets,
    file_targets: &tracing_subscriber::filter::Targets,
    file_router: MultiFileRouter,
    #[cfg_attr(not(feature = "otel"), allow(unused_variables))] otel_layer: Option<OtelLayer>,
) {
    use tracing_subscriber::{EnvFilter, Registry, fmt, layer::SubscriberExt};

    // RUST_LOG acts as a global upper-bound for console/file if present.
    // If not set, we don't clamp here — YAML targets drive levels.
    let env: Option<EnvFilter> = EnvFilter::try_from_default_env().ok();

    // Console writer (non-blocking stderr)
    let (nb_stderr, guard) = tracing_appender::non_blocking(std::io::stderr());
    let _ = CONSOLE_GUARD.set(guard);

    // Console fmt layer (human-friendly)
    let console_layer = fmt::layer()
        .with_writer(nb_stderr)
        .with_ansi(std::io::stderr().is_terminal())
        .with_target(true)
        .with_level(true)
        .with_timer(fmt::time::UtcTime::rfc_3339())
        .with_filter(console_targets.clone());

    // File fmt layer (JSON) if router is not empty
    let file_layer_opt = if file_router.is_empty() {
        None
    } else {
        Some(
            fmt::layer()
                .json()
                .with_ansi(false)
                .with_target(true)
                .with_level(true)
                .with_timer(fmt::time::UtcTime::rfc_3339())
                .with_writer(file_router)
                .with_filter(file_targets.clone()),
        )
    };

    // Build subscriber:
    // 1) OTEL first (because your OtelLayer is bound to `Registry`);
    //    also filter OTEL by the SAME console targets from YAML.
    // 2) Then EnvFilter (caps console/file if RUST_LOG is set).
    // 3) Then console + file fmt layers.
    let subscriber = {
        let base = Registry::default();

        #[cfg(feature = "otel")]
        let base = {
            let otel_opt = otel_layer.map(|otel| otel.with_filter(console_targets.clone()));
            base.with(otel_opt)
        };
        #[cfg(not(feature = "otel"))]
        let base = base;

        let base = base.with(env);
        base.with(console_layer).with(file_layer_opt)
    };

    let _ = subscriber.try_init();
}

fn init_minimal(
    #[cfg_attr(not(feature = "otel"), allow(unused_variables))] otel: Option<OtelLayer>,
) {
    use tracing_subscriber::{EnvFilter, Registry, fmt, layer::SubscriberExt};

    // If RUST_LOG is set, it will cap fmt output; otherwise don't clamp here.
    let env = EnvFilter::try_from_default_env().ok();

    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_timer(fmt::time::UtcTime::rfc_3339());

    // Same ordering: OTEL (if any) first, then EnvFilter, then fmt.
    let subscriber = {
        let base = Registry::default();

        #[cfg(feature = "otel")]
        let base = base.with(otel);
        #[cfg(not(feature = "otel"))]
        let base = base;

        base.with(env).with(fmt_layer)
    };

    let _ = subscriber.try_init();
}
