//! Cross-platform filesystem utilities.
//!
//! Provides helpers for path resolution, directory creation, and
//! bidirectional Windows/WSL path translation.
//!
//! # Example
//!
//! ```rust
//! use commons::fs::{resolve_path, ensure_dir, to_wsl_path};
//! use std::path::Path;
//!
//! let expanded = resolve_path("~/config.toml");
//! assert!(!expanded.starts_with("~"));
//!
//! let wsl = to_wsl_path(r"C:\Users\Name\file.txt");
//! assert_eq!(wsl, Path::new("/mnt/c/Users/Name/file.txt"));
//! ```

use std::io;
use std::path::{Path, PathBuf};

/// Expand `~` to the current user's home directory.
///
/// Only `~/...` (and bare `~`) are expanded. Patterns like `~otheruser/...`
/// are **not** supported — they require OS-specific user lookups that are
/// outside this crate's scope — and are returned unchanged.
///
/// Falls back to returning the path unchanged if the home directory
/// cannot be determined.
#[must_use]
pub fn resolve_path(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    let path_str = path.to_string_lossy();

    if !path_str.starts_with('~') {
        return path.to_path_buf();
    }

    // Only expand bare `~` or `~/...` / `~\...`.  Anything else (e.g.
    // `~otheruser/...`) is returned as-is because we cannot resolve
    // arbitrary user home directories portably.
    if path_str.len() > 1 && !path_str[1..].starts_with('/') && !path_str[1..].starts_with('\\') {
        return path.to_path_buf();
    }

    let Some(home) = home_dir() else {
        return path.to_path_buf();
    };

    if path_str == "~" {
        return home;
    }

    // Strip the `~/` or `~\` prefix.
    let rest = &path_str[2..];
    home.join(rest)
}

/// Create a directory and all of its parents if they don't exist.
///
/// Equivalent to `std::fs::create_dir_all` but reads more clearly
/// at call sites.
///
/// # Errors
///
/// Returns an `io::Error` if directory creation fails (e.g., permission denied).
pub fn ensure_dir(path: impl AsRef<Path>) -> io::Result<()> {
    std::fs::create_dir_all(path)
}

/// Detect whether the current process is running under WSL.
///
/// Checks `/proc/version` for the string `microsoft` (case-insensitive),
/// which is present in both WSL 1 and WSL 2 kernels.
/// Always returns `false` on non-Linux platforms.
#[must_use]
#[cfg(target_os = "linux")]
pub fn is_wsl() -> bool {
    std::fs::read_to_string("/proc/version")
        .is_ok_and(|v| v.to_ascii_lowercase().contains("microsoft"))
}

/// Detect whether the current process is running under WSL.
///
/// Always returns `false` on non-Linux platforms.
#[must_use]
#[cfg(not(target_os = "linux"))]
pub const fn is_wsl() -> bool {
    false
}

/// Convert a Windows-style path to its WSL `/mnt/` equivalent.
///
/// Handles drive letters (`C:\...` or `C:/...`) by lowercasing the drive
/// letter and mapping to `/mnt/<drive>/...`. Forward and backward slashes
/// in the input are both supported.
///
/// Paths that don't match `X:\` or `X:/` patterns are returned unchanged
/// (with backslashes normalised to forward slashes).
#[must_use]
pub fn to_wsl_path(path: impl AsRef<Path>) -> PathBuf {
    let s = path.as_ref().to_string_lossy();

    // Check for drive-letter pattern: single ASCII alpha followed by :\ or :/
    let bytes = s.as_bytes();
    if bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes[2] == b'\\' || bytes[2] == b'/')
    {
        let drive = (bytes[0] as char).to_ascii_lowercase();
        let rest = s[3..].replace('\\', "/");
        return PathBuf::from(format!("/mnt/{drive}/{rest}"));
    }

    // Not a Windows path — normalise slashes only
    PathBuf::from(s.replace('\\', "/"))
}

/// Convert a WSL `/mnt/` path back to a Windows-style path.
///
/// Maps `/mnt/c/Users/...` to `C:\Users\...`. The drive letter is
/// uppercased and forward slashes are converted to backslashes.
///
/// Only single-letter mount points are converted (matching WSL's
/// standard drive-letter mounts). Multi-letter mounts such as
/// `/mnt/wslg/` or `/mnt/data/` are left unchanged.
///
/// Paths that don't start with `/mnt/<letter>/` are returned unchanged.
#[must_use]
pub fn from_wsl_path(path: impl AsRef<Path>) -> PathBuf {
    let s = path.as_ref().to_string_lossy();

    if s.starts_with("/mnt/") && s.len() >= 7 {
        let bytes = s.as_bytes();
        // Require exactly one ASCII letter followed by '/' — this
        // prevents multi-letter mounts like /mnt/wslg/ from being
        // misinterpreted as drive letters.
        if bytes[5].is_ascii_alphabetic() && bytes[6] == b'/' {
            let drive = (bytes[5] as char).to_ascii_uppercase();
            let rest = s[7..].replace('/', "\\");
            return PathBuf::from(format!("{drive}:\\{rest}"));
        }
    }

    PathBuf::from(s.into_owned())
}

/// Get the current user's home directory.
#[cfg(unix)]
fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

/// Get the current user's home directory.
#[cfg(windows)]
fn home_dir() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_path_tilde() {
        let home = home_dir().unwrap();
        assert_eq!(resolve_path("~"), home);
        assert_eq!(resolve_path("~/config.toml"), home.join("config.toml"));
        assert_eq!(
            resolve_path("~/.euxis/config.toml"),
            home.join(".euxis/config.toml")
        );
    }

    #[test]
    fn test_resolve_path_no_tilde() {
        let p = Path::new("/usr/local/bin");
        assert_eq!(resolve_path(p), p.to_path_buf());

        let rel = Path::new("relative/path");
        assert_eq!(resolve_path(rel), rel.to_path_buf());
    }

    #[test]
    fn test_ensure_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let nested = tmp.path().join("a").join("b").join("c");
        assert!(!nested.exists());
        ensure_dir(&nested).unwrap();
        assert!(nested.is_dir());
    }

    #[test]
    fn test_to_wsl_path_drive_letters() {
        assert_eq!(
            to_wsl_path(r"C:\Users\Name\file.txt"),
            PathBuf::from("/mnt/c/Users/Name/file.txt")
        );
        assert_eq!(
            to_wsl_path(r"D:\Projects\src"),
            PathBuf::from("/mnt/d/Projects/src")
        );
        // Forward-slash variant
        assert_eq!(
            to_wsl_path("E:/data/log.txt"),
            PathBuf::from("/mnt/e/data/log.txt")
        );
    }

    #[test]
    fn test_to_wsl_path_non_windows() {
        // Unix paths pass through unchanged
        assert_eq!(
            to_wsl_path("/usr/local/bin"),
            PathBuf::from("/usr/local/bin")
        );
        assert_eq!(to_wsl_path("relative/path"), PathBuf::from("relative/path"));
    }

    #[test]
    fn test_resolve_path_tilde_otheruser() {
        // ~otheruser should NOT be expanded — returned unchanged
        let p = Path::new("~otheruser/downloads");
        assert_eq!(resolve_path(p), p.to_path_buf());
    }

    #[test]
    fn test_from_wsl_path_drive_letters() {
        assert_eq!(
            from_wsl_path("/mnt/c/Users/Name/file.txt"),
            PathBuf::from("C:\\Users\\Name\\file.txt")
        );
        assert_eq!(
            from_wsl_path("/mnt/d/Projects/src"),
            PathBuf::from("D:\\Projects\\src")
        );
    }

    #[test]
    fn test_from_wsl_path_passthrough() {
        // Non-WSL paths pass through unchanged
        assert_eq!(
            from_wsl_path("/usr/local/bin"),
            PathBuf::from("/usr/local/bin")
        );
        // Multi-letter mounts are NOT drive letters
        assert_eq!(
            from_wsl_path("/mnt/data/shared"),
            PathBuf::from("/mnt/data/shared")
        );
        assert_eq!(
            from_wsl_path("/mnt/wslg/x.socket"),
            PathBuf::from("/mnt/wslg/x.socket")
        );
        assert_eq!(
            from_wsl_path("/mnt/cache/app"),
            PathBuf::from("/mnt/cache/app")
        );
    }

    #[test]
    fn test_wsl_roundtrip() {
        let win = r"C:\Users\Name\file.txt";
        let wsl = to_wsl_path(win);
        let back = from_wsl_path(&wsl);
        assert_eq!(back, PathBuf::from(win));
    }

    #[test]
    fn test_is_wsl_smoke() {
        // Just ensure it doesn't panic; actual value depends on runtime.
        let _ = is_wsl();
    }
}
