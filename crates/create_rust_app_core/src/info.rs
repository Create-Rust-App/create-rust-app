//! Environment diagnostics printed by `--info`.

/// Short brand banner (ASCII, no emoji — mirrors the sibling UX rule).
pub fn brand_banner() -> &'static str {
    "create-rust-app — scaffold Rust projects"
}

/// One-shot environment summary for bug reports.
pub fn env_info() -> String {
    format!(
        "create-rust-app {}\nos={} arch={}\ncatalog={}\ncache={}",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH,
        create_rust_app_catalog_default(),
        crate::cache::cache_dir().to_string_lossy(),
    )
}

fn create_rust_app_catalog_default() -> &'static str {
    crate::catalog::DEFAULT_CATALOG_URL
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn banner_names_the_binary() {
        assert!(brand_banner().contains("create-rust-app"));
    }

    #[test]
    fn info_mentions_platform() {
        let info = env_info();
        assert!(info.contains(std::env::consts::OS));
        assert!(info.contains(std::env::consts::ARCH));
    }
}
