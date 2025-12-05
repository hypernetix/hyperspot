use std::{
    env, fs,
    path::{Path, PathBuf},
};

/// Errors for resolving the home directory
#[derive(Debug, thiserror::Error)]
pub enum HomeDirError {
    #[error("HOME environment variable is not set")]
    HomeMissing,
    #[error("APPDATA environment variable is not set")]
    AppDataMissing,
    #[error("home_dir must be an absolute path on Windows: {0}")]
    WindowsAbsoluteRequired(String),
    #[error("home_dir must be an absolute path (after ~ expansion): {0}")]
    AbsoluteRequired(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Normalize and resolve the home directory path based on platform rules.
///
/// Rules:
/// - If `config_home` is provided:
///   - Windows: support `~` expansion to the user profile; the final path must be absolute.
///   - Linux/macOS: allow `~` expansion; the final path must be absolute.
/// - If `config_home` is not provided:
///   - Windows: use `%APPDATA%/<default_subdir>` (error if `APPDATA` is missing).
///   - Linux/macOS: use `$HOME/<default_subdir>` (error if `HOME` is missing).
///
/// If `create` is true, the directory is created if missing.
///
/// `default_subdir` is usually ".hyperspot", but can be customized by the caller.
pub fn resolve_home_dir(
    config_home: Option<String>,
    default_subdir: &str,
    create: bool,
) -> Result<PathBuf, HomeDirError> {
    #[cfg(target_os = "windows")]
    {
        let path = if let Some(raw) = config_home {
            // On Windows, support ~ expansion to the user profile, and require absolute after expansion.
            let expanded: PathBuf = if raw.starts_with('~') {
                let user_home = env::var("USERPROFILE")
                    .or_else(|_| env::var("HOME"))
                    .map_err(|_| HomeDirError::WindowsAbsoluteRequired(raw.clone()))?;
                if raw == "~" {
                    PathBuf::from(user_home)
                } else if let Some(rest) =
                    raw.strip_prefix("~/").or_else(|| raw.strip_prefix("~\\"))
                {
                    Path::new(&user_home).join(rest)
                } else {
                    // Patterns like "~username" are not supported; fallback to treating as user home join the rest
                    let rest = raw.trim_start_matches('~');
                    let rest = rest.trim_start_matches(['/', '\\']);
                    Path::new(&user_home).join(rest)
                }
            } else {
                PathBuf::from(&raw)
            };

            if !expanded.is_absolute() {
                return Err(HomeDirError::WindowsAbsoluteRequired(raw));
            }
            expanded
        } else {
            // Default to %APPDATA%/<default_subdir>
            let appdata = env::var("APPDATA").map_err(|_| HomeDirError::AppDataMissing)?;
            Path::new(&appdata).join(default_subdir)
        };

        if create {
            fs::create_dir_all(&path)?;
        }
        Ok(path)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let path = if let Some(raw) = config_home {
            // Allow ~ expansion at the beginning
            let expanded = if let Some(stripped) = raw.strip_prefix("~/") {
                let home = env::home_dir().ok_or(HomeDirError::HomeMissing)?;
                Path::new(&home).join(stripped)
            } else if raw == "~" {
                env::home_dir().ok_or(HomeDirError::HomeMissing)?
            } else {
                PathBuf::from(raw.clone())
            };

            if !expanded.is_absolute() {
                return Err(HomeDirError::AbsoluteRequired(
                    expanded.to_string_lossy().into(),
                ));
            }
            expanded
        } else {
            // Default to $HOME/<default_subdir>
            let home = env::home_dir().ok_or(HomeDirError::HomeMissing)?;
            Path::new(&home).join(default_subdir)
        };

        if create {
            fs::create_dir_all(&path)?;
        }
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::tempdir;
    #[test]
    fn tests_secuential() {
        // we can't run this in parallel as they interfere with each other
        #[cfg(target_os = "windows")]
        {
            windows_absolute_path_ok();
            windows_relative_path_error();
            windows_default_uses_appdata();
            windows_error_when_appdata_missing();
            windows_creates_directory_when_flag_true();
        }
        #[cfg(not(target_os = "windows"))]
        {
            unix_resolve_with_tilde();
            unix_resolve_with_only_tilde();
            unix_resolve_default_home_dir();
            unix_resolve_absolute_path_ok();
            unix_resolve_relative_path_error();
            unix_resolve_creates_directory();
            unix_will_fallback_when_home_missing();
        }
    }

    /// Helper: path must be absolute and not start with '~'.
    #[cfg(not(target_os = "windows"))]
    fn is_normalized(path: &std::path::Path) -> bool {
        path.is_absolute() && !path.to_string_lossy().starts_with('~')
    }

    // -------------------------
    // Unix/macOS test suite
    // -------------------------
    #[cfg(not(target_os = "windows"))]
    fn unix_resolve_with_tilde() {
        // Fake HOME for the test
        let tmp = tempdir().unwrap();
        env::set_var("HOME", tmp.path());

        let result = resolve_home_dir(Some("~/myapp".into()), ".hyperspot", false).unwrap();

        assert!(is_normalized(&result));
        assert!(result.ends_with("myapp"));
    }

    #[cfg(not(target_os = "windows"))]
    fn unix_resolve_with_only_tilde() {
        let tmp = tempdir().unwrap();
        env::set_var("HOME", tmp.path());

        let result = resolve_home_dir(Some("~".into()), ".hyperspot", false).unwrap();

        assert!(is_normalized(&result));
        assert_eq!(result, tmp.path());
    }

    #[cfg(not(target_os = "windows"))]
    fn unix_resolve_default_home_dir() {
        let tmp = tempdir().unwrap();
        env::set_var("HOME", tmp.path());

        let result = resolve_home_dir(None, ".hyperspot", false).unwrap();

        assert!(is_normalized(&result));
        assert!(result.ends_with(".hyperspot"));
    }

    #[cfg(not(target_os = "windows"))]
    fn unix_resolve_absolute_path_ok() {
        let tmp = tempdir().unwrap();
        let abs_path = tmp.path().join("custom_dir");

        let result = resolve_home_dir(
            Some(abs_path.to_string_lossy().to_string()),
            ".hyperspot",
            false,
        )
        .unwrap();

        assert_eq!(result, abs_path);
    }

    #[cfg(not(target_os = "windows"))]
    fn unix_resolve_relative_path_error() {
        // Relative path is not allowed on Unix after expansion
        let err = resolve_home_dir(Some("relative/path".into()), ".hyperspot", false).unwrap_err();
        match err {
            HomeDirError::AbsoluteRequired(_) => {}
            _ => panic!("Expected AbsoluteRequired, got {:?}", err),
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn unix_resolve_creates_directory() {
        let tmp = tempdir().unwrap();
        env::set_var("HOME", tmp.path());
        let target = tmp.path().join(".hyperspot");

        // Directory should be created
        let result = resolve_home_dir(None, ".hyperspot", true).unwrap();
        assert!(result.exists());
        assert_eq!(result, target);
    }

    #[cfg(not(target_os = "windows"))]
    fn unix_will_fallback_when_home_missing() {
        // Save and restore original HOME to isolate this test
        let original_home = env::var("HOME").ok();
        env::remove_var("HOME");

        let result = resolve_home_dir(None, ".hyperspot", false);

        // Restore original HOME before asserting to avoid affecting other tests
        if let Some(home) = original_home {
            env::set_var("HOME", home);
        }

        assert!(result.is_ok());
    }

    // -------------------------
    // Windows test suite
    // -------------------------
    #[cfg(target_os = "windows")]
    fn windows_absolute_path_ok() {
        // On Windows, only absolute paths are accepted when provided.
        let tmp = tempdir().unwrap();
        let abs_path = tmp.path().join("custom_dir");

        let result = resolve_home_dir(
            Some(abs_path.to_string_lossy().to_string()),
            ".hyperspot",
            false,
        )
        .unwrap();

        assert_eq!(result, abs_path);
        assert!(result.is_absolute());
    }

    #[cfg(target_os = "windows")]
    fn windows_relative_path_error() {
        // On Windows, a provided path must be absolute (no ~, no relative).
        let err = resolve_home_dir(Some("relative\\path".into()), ".hyperspot", false).unwrap_err();
        match err {
            HomeDirError::WindowsAbsoluteRequired(s) => {
                assert!(s.contains("relative\\path"));
            }
            _ => panic!("Expected WindowsAbsoluteRequired, got {:?}", err),
        }
    }

    #[cfg(target_os = "windows")]
    fn windows_default_uses_appdata() {
        // When not provided, it must use %APPDATA%\<default_subdir>.
        let tmp = tempdir().unwrap();
        env::set_var("APPDATA", tmp.path());

        let result = resolve_home_dir(None, ".hyperspot", false).unwrap();

        assert!(result.is_absolute());
        assert!(result.ends_with(".hyperspot"));
        assert!(result.starts_with(tmp.path()));
    }

    #[cfg(target_os = "windows")]
    fn windows_error_when_appdata_missing() {
        env::remove_var("APPDATA");

        let err = resolve_home_dir(None, ".hyperspot", false).unwrap_err();
        match err {
            HomeDirError::AppDataMissing => {}
            _ => panic!("Expected AppDataMissing, got {:?}", err),
        }
    }

    #[cfg(target_os = "windows")]
    fn windows_creates_directory_when_flag_true() {
        let tmp = tempdir().unwrap();
        env::set_var("APPDATA", tmp.path());
        let target = tmp.path().join(".hyperspot");

        let result = resolve_home_dir(None, ".hyperspot", true).unwrap();
        assert!(result.exists());
        assert_eq!(result, target);
    }
}
