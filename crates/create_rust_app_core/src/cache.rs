//! Local catalog cache helpers (`cache clean | status | path`).

use std::fs;
use std::path::PathBuf;

/// Resolve the cache directory.
///
/// `CRA_CACHE_DIR` wins when set; otherwise a platform default is used.
pub fn cache_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("CRA_CACHE_DIR") {
        if !dir.is_empty() {
            return PathBuf::from(dir);
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home)
                .join("Library")
                .join("Caches")
                .join("create-rust-app");
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            return PathBuf::from(local).join("create-rust-app").join("cache");
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".cache").join("create-rust-app");
    }
    PathBuf::from(".cache").join("create-rust-app")
}

/// Remove the cache directory (missing directories are a no-op).
pub fn clean_cache() -> std::io::Result<()> {
    let dir = cache_dir();
    if dir.exists() {
        fs::remove_dir_all(&dir)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn respects_cache_dir_override() {
        let key = "CRA_CACHE_DIR";
        let previous = std::env::var(key).ok();
        std::env::set_var(key, "/tmp/cra-cache-test");
        assert_eq!(cache_dir(), PathBuf::from("/tmp/cra-cache-test"));
        match previous {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
    }

    #[test]
    fn cleaning_missing_cache_is_noop() {
        let key = "CRA_CACHE_DIR";
        let previous = std::env::var(key).ok();
        let dir = tempfile::tempdir().expect("tempdir");
        let missing = dir.path().join("absent");
        std::env::set_var(key, &missing);
        clean_cache().expect("noop");
        match previous {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
    }
}
