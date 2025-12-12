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
    #[error("relative paths with directory separators are not allowed: {0}")]
    RelativePathNotAllowed(String),
    #[error("failed to get executable path: {0}")]
    ExecutablePathError(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Expand `~` prefix to user home directory.
///
/// Returns the path unchanged if no tilde prefix is present.
/// On Windows, uses `USERPROFILE` or `HOME` environment variable.
/// On Unix, uses `HOME` environment variable.
pub fn expand_tilde(raw: &str) -> Result<PathBuf, HomeDirError> {
    #[cfg(target_os = "windows")]
    {
        if raw.starts_with('~') {
            let user_home = env::var("USERPROFILE")
                .or_else(|_| env::var("HOME"))
                .map_err(|_| HomeDirError::HomeMissing)?;
            if raw == "~" {
                Ok(PathBuf::from(user_home))
            } else if let Some(rest) = raw.strip_prefix("~/").or_else(|| raw.strip_prefix("~\\")) {
                Ok(Path::new(&user_home).join(rest))
            } else {
                // Patterns like "~username" are not supported; treat as user home + rest
                let rest = raw.trim_start_matches('~');
                let rest = rest.trim_start_matches(['/', '\\']);
                Ok(Path::new(&user_home).join(rest))
            }
        } else {
            Ok(PathBuf::from(raw))
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Some(stripped) = raw.strip_prefix("~/") {
            let home = env::var("HOME").map_err(|_| HomeDirError::HomeMissing)?;
            Ok(Path::new(&home).join(stripped))
        } else if raw == "~" {
            let home = env::var("HOME").map_err(|_| HomeDirError::HomeMissing)?;
            Ok(PathBuf::from(home))
        } else {
            Ok(PathBuf::from(raw))
        }
    }
}

/// Normalize an executable path for OoP modules.
///
/// Rules:
/// - `~` prefix: expand to user home directory
/// - Absolute path: use as-is
/// - Filename only (no path separators): prepend directory where executable of current process lives
/// - Relative path with separators: error (ambiguous)
pub fn normalize_executable_path(raw: &str) -> Result<PathBuf, HomeDirError> {
    // First, expand tilde if present
    let expanded = expand_tilde(raw)?;

    // If already absolute, return as-is
    if expanded.is_absolute() {
        return Ok(expanded);
    }

    // Check if it's just a filename (no path separators)
    let has_separator = raw.contains('/') || raw.contains('\\');

    if !has_separator {
        // Filename only - prepend directory where main executable lives
        let exe_path =
            env::current_exe().map_err(|e| HomeDirError::ExecutablePathError(e.to_string()))?;
        let exe_dir = exe_path.parent().ok_or_else(|| {
            HomeDirError::ExecutablePathError("executable has no parent directory".to_string())
        })?;
        Ok(exe_dir.join(&expanded))
    } else {
        // Relative path with separators - not allowed
        Err(HomeDirError::RelativePathNotAllowed(raw.to_string()))
    }
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
            // Expand tilde and require absolute path
            let expanded = expand_tilde(&raw)?;

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
            // Expand tilde and require absolute path
            let expanded = expand_tilde(&raw)?;

            if !expanded.is_absolute() {
                return Err(HomeDirError::AbsoluteRequired(
                    expanded.to_string_lossy().into(),
                ));
            }
            expanded
        } else {
            // Default to $HOME/<default_subdir>
            let home = env::var("HOME").map_err(|_| HomeDirError::HomeMissing)?;
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
    use tempfile::tempdir;

    /// Helper: path must be absolute and not start with '~'.
    #[cfg(not(target_os = "windows"))]
    fn is_normalized(path: &std::path::Path) -> bool {
        path.is_absolute() && !path.to_string_lossy().starts_with('~')
    }

    // -------------------------
    // Unix/macOS test suite
    // -------------------------
    #[test]
    #[cfg(not(target_os = "windows"))]
    fn unix_resolve_with_tilde() {
        let tmp = tempdir().unwrap();
        let tmp_path = tmp.path().to_str().unwrap();

        temp_env::with_var("HOME", Some(tmp_path), || {
            let result = resolve_home_dir(Some("~/myapp".into()), ".hyperspot", false).unwrap();

            assert!(is_normalized(&result));
            assert!(result.ends_with("myapp"));
        });
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn unix_resolve_with_only_tilde() {
        let tmp = tempdir().unwrap();
        let tmp_path = tmp.path().to_str().unwrap();

        temp_env::with_var("HOME", Some(tmp_path), || {
            let result = resolve_home_dir(Some("~".into()), ".hyperspot", false).unwrap();

            assert!(is_normalized(&result));
            assert_eq!(result, tmp.path());
        });
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn unix_resolve_default_home_dir() {
        let tmp = tempdir().unwrap();
        let tmp_path = tmp.path().to_str().unwrap();

        temp_env::with_var("HOME", Some(tmp_path), || {
            let result = resolve_home_dir(None, ".hyperspot", false).unwrap();

            assert!(is_normalized(&result));
            assert!(result.ends_with(".hyperspot"));
        });
    }

    #[test]
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

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn unix_resolve_relative_path_error() {
        // Relative path is not allowed on Unix after expansion
        let err = resolve_home_dir(Some("relative/path".into()), ".hyperspot", false).unwrap_err();
        match err {
            HomeDirError::AbsoluteRequired(_) => {}
            _ => panic!("Expected AbsoluteRequired, got {:?}", err),
        }
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn unix_resolve_creates_directory() {
        let tmp = tempdir().unwrap();
        let tmp_path = tmp.path().to_str().unwrap();
        let target = tmp.path().join(".hyperspot");

        temp_env::with_var("HOME", Some(tmp_path), || {
            let result = resolve_home_dir(None, ".hyperspot", true).unwrap();
            assert!(result.exists());
            assert_eq!(result, target);
        });
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn unix_error_when_home_missing() {
        temp_env::with_var_unset("HOME", || {
            let err = resolve_home_dir(None, ".hyperspot", false).unwrap_err();
            match err {
                HomeDirError::HomeMissing => {}
                _ => panic!("Expected HomeMissing, got {:?}", err),
            }
        });
    }

    // -------------------------
    // Windows test suite
    // -------------------------
    #[test]
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

    #[test]
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

    #[test]
    #[cfg(target_os = "windows")]
    fn windows_default_uses_appdata() {
        let tmp = tempdir().unwrap();
        let tmp_path = tmp.path().to_str().unwrap();

        temp_env::with_var("APPDATA", Some(tmp_path), || {
            let result = resolve_home_dir(None, ".hyperspot", false).unwrap();

            assert!(result.is_absolute());
            assert!(result.ends_with(".hyperspot"));
            assert!(result.starts_with(tmp.path()));
        });
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn windows_error_when_appdata_missing() {
        temp_env::with_var_unset("APPDATA", || {
            let err = resolve_home_dir(None, ".hyperspot", false).unwrap_err();
            match err {
                HomeDirError::AppDataMissing => {}
                _ => panic!("Expected AppDataMissing, got {:?}", err),
            }
        });
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn windows_creates_directory_when_flag_true() {
        let tmp = tempdir().unwrap();
        let tmp_path = tmp.path().to_str().unwrap();
        let target = tmp.path().join(".hyperspot");

        temp_env::with_var("APPDATA", Some(tmp_path), || {
            let result = resolve_home_dir(None, ".hyperspot", true).unwrap();
            assert!(result.exists());
            assert_eq!(result, target);
        });
    }

    // -------------------------
    // expand_tilde tests
    // -------------------------
    #[test]
    #[cfg(not(target_os = "windows"))]
    fn expand_tilde_with_path() {
        let tmp = tempdir().unwrap();
        let tmp_path = tmp.path().to_str().unwrap();

        temp_env::with_var("HOME", Some(tmp_path), || {
            let result = super::expand_tilde("~/bin/app").unwrap();
            assert!(result.is_absolute());
            assert!(result.ends_with("bin/app"));
        });
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn expand_tilde_only() {
        let tmp = tempdir().unwrap();
        let tmp_path = tmp.path().to_str().unwrap();

        temp_env::with_var("HOME", Some(tmp_path), || {
            let result = super::expand_tilde("~").unwrap();
            assert_eq!(result, tmp.path());
        });
    }

    #[test]
    fn expand_tilde_no_tilde() {
        let result = super::expand_tilde("/usr/bin/app").unwrap();
        assert_eq!(result, PathBuf::from("/usr/bin/app"));
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn windows_expand_tilde_with_path() {
        let tmp = tempdir().unwrap();
        let tmp_path = tmp.path().to_str().unwrap();

        temp_env::with_var("USERPROFILE", Some(tmp_path), || {
            let result = super::expand_tilde("~/bin/app").unwrap();
            assert!(result.is_absolute());
            assert!(result.ends_with("bin\\app") || result.ends_with("bin/app"));
        });
    }

    // -------------------------
    // normalize_executable_path tests
    // -------------------------
    #[test]
    #[cfg(not(target_os = "windows"))]
    fn normalize_exec_absolute_path() {
        let result = super::normalize_executable_path("/usr/bin/myapp").unwrap();
        assert_eq!(result, PathBuf::from("/usr/bin/myapp"));
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn windows_normalize_exec_absolute_path() {
        let result = super::normalize_executable_path("C:\\bin\\myapp.exe").unwrap();
        assert_eq!(result, PathBuf::from("C:\\bin\\myapp.exe"));
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn normalize_exec_tilde_path() {
        let tmp = tempdir().unwrap();
        let tmp_path = tmp.path().to_str().unwrap();

        temp_env::with_var("HOME", Some(tmp_path), || {
            let result = super::normalize_executable_path("~/bin/myapp").unwrap();
            assert!(result.is_absolute());
            assert!(result.ends_with("bin/myapp"));
        });
    }

    #[test]
    fn normalize_exec_filename_only() {
        // A bare filename should be prepended with the executable's directory
        let result = super::normalize_executable_path("myapp.exe").unwrap();
        let exe_dir = env::current_exe().unwrap().parent().unwrap().to_path_buf();
        assert_eq!(result, exe_dir.join("myapp.exe"));
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn normalize_exec_relative_path_error() {
        // Relative paths with separators should error
        let err = super::normalize_executable_path("./bin/myapp").unwrap_err();
        match err {
            HomeDirError::RelativePathNotAllowed(s) => {
                assert!(s.contains("./bin/myapp"));
            }
            _ => panic!("Expected RelativePathNotAllowed, got {:?}", err),
        }
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn windows_normalize_exec_relative_path_error() {
        // Relative paths with separators should error
        let err = super::normalize_executable_path(".\\bin\\myapp").unwrap_err();
        match err {
            HomeDirError::RelativePathNotAllowed(s) => {
                assert!(s.contains(".\\bin\\myapp"));
            }
            _ => panic!("Expected RelativePathNotAllowed, got {:?}", err),
        }
    }
}
