//! Template catalog loading, filtering, and formatting.

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::scaffold::EngineError;

/// Default catalog URL served by the `cra-templates` bank.
///
/// Override with `--catalog-path`, `--catalog-url`, or a fork for local
/// testing (`file://` URLs are read from disk).
pub const DEFAULT_CATALOG_URL: &str =
    "https://raw.githubusercontent.com/Create-Rust-App/cra-templates/main/templates.json";

/// One scaffoldable project template.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateEntry {
    /// Stable slug used with `--template` (for example `axum-starter`).
    pub slug: String,
    /// One-line human description.
    #[serde(default)]
    pub description: String,
    /// Category tags used by `--category` filtering.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Where template content lives: `file://` directory, git URL
    /// (with optional `?subdir=`), or empty when unknown.
    #[serde(default)]
    pub url: String,
}

/// One composable extension applied on top of a template.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddonEntry {
    /// Stable slug used with `--addons` (for example `all-github-setup`).
    pub slug: String,
    /// One-line human description.
    #[serde(default)]
    pub description: String,
    /// Overlay source: `file://` directory, git URL (with optional
    /// `?subdir=`), or empty when unknown.
    #[serde(default)]
    pub url: String,
}

/// The full template/addon bank served as `templates.json`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Catalog {
    /// Available project templates.
    #[serde(default)]
    pub templates: Vec<TemplateEntry>,
    /// Available addons. The official bank serves these under the
    /// `extensions` key, so both spellings deserialize here.
    #[serde(default, alias = "extensions")]
    pub addons: Vec<AddonEntry>,
}

/// Resolve the local fixture catalog path.
///
/// Mirrors the sibling `--fixture` / `--fixture-dir` behaviour: with no
/// directory given, falls back to `CRA_FIXTURE_DIR` and finally to the
/// repo-local `fixtures/catalog` directory.
pub fn resolve_fixture_catalog_path(dir: Option<&str>) -> PathBuf {
    let base = match dir {
        Some(d) if !d.is_empty() => PathBuf::from(d),
        _ => match std::env::var("CRA_FIXTURE_DIR") {
            Ok(d) if !d.is_empty() => PathBuf::from(d),
            _ => PathBuf::from("fixtures/catalog"),
        },
    };
    base.join("templates.json")
}

fn strip_file_scheme(location: &str) -> Option<&str> {
    location.strip_prefix("file://")
}

fn read_local_catalog(path: &str) -> Result<Catalog, EngineError> {
    let raw = fs::read_to_string(path).map_err(|source| EngineError::CatalogRead {
        path: path.to_string(),
        source,
    })?;
    parse_catalog(&raw)
}

fn parse_catalog(raw: &str) -> Result<Catalog, EngineError> {
    serde_json::from_str(raw).map_err(EngineError::CatalogParse)
}

fn fetch_remote_catalog(url: &str) -> Result<Catalog, EngineError> {
    let response = ureq::get(url)
        .set("Accept", "application/json")
        .set("User-Agent", "create-rust-app")
        .timeout(std::time::Duration::from_secs(30))
        .call()
        .map_err(|source| EngineError::CatalogFetch {
            url: url.to_string(),
            source: Box::new(source),
        })?;
    let raw = response
        .into_string()
        .map_err(|source| EngineError::CatalogFetchBody {
            url: url.to_string(),
            source,
        })?;
    parse_catalog(&raw)
}

/// Load the catalog, preferring an explicit local path.
///
/// Resolution order: `catalog_path` (local file) → `catalog_url`
/// (`file://` URLs are read from disk, anything else is fetched over
/// HTTPS) → [`DEFAULT_CATALOG_URL`].
pub fn load_catalog(
    catalog_path: Option<&str>,
    catalog_url: Option<&str>,
) -> Result<Catalog, EngineError> {
    if let Some(path) = catalog_path {
        if !path.is_empty() {
            let local = strip_file_scheme(path).unwrap_or(path);
            return read_local_catalog(local);
        }
    }
    let url = match catalog_url {
        Some(url) if !url.is_empty() => url.to_string(),
        _ => DEFAULT_CATALOG_URL.to_string(),
    };
    if let Some(local) = strip_file_scheme(&url) {
        return read_local_catalog(local);
    }
    fetch_remote_catalog(&url)
}

/// Sorted template slugs in catalog order.
pub fn list_template_names(catalog: &Catalog) -> Vec<String> {
    catalog
        .templates
        .iter()
        .map(|entry| entry.slug.clone())
        .collect()
}

/// Sorted addon slugs in catalog order.
pub fn list_addon_names(catalog: &Catalog) -> Vec<String> {
    catalog
        .addons
        .iter()
        .map(|entry| entry.slug.clone())
        .collect()
}

/// Template slugs whose tags contain `category`.
pub fn list_template_names_in_category(catalog: &Catalog, category: &str) -> Vec<String> {
    catalog
        .templates
        .iter()
        .filter(|entry| entry.tags.iter().any(|tag| tag == category))
        .map(|entry| entry.slug.clone())
        .collect()
}

/// Render a `Title:\n  - name` list for terminal output.
pub fn format_catalog_list(title: &str, names: &[String]) -> String {
    let mut out = format!("{title}:");
    for name in names {
        out.push_str(&format!("\n  - {name}"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_catalog() -> Catalog {
        Catalog {
            templates: vec![
                TemplateEntry {
                    slug: "web-server".to_string(),
                    description: "Axum web server".to_string(),
                    tags: vec!["web".to_string()],
                    url: "file:///bank/web-server".to_string(),
                },
                TemplateEntry {
                    slug: "cli".to_string(),
                    description: "Clap CLI starter".to_string(),
                    tags: vec!["tooling".to_string()],
                    url: String::new(),
                },
            ],
            addons: vec![AddonEntry {
                slug: "github-setup".to_string(),
                description: "CI workflows".to_string(),
                url: "file:///bank/github-setup".to_string(),
            }],
        }
    }

    #[test]
    fn lists_templates_and_addons_in_order() {
        let catalog = sample_catalog();
        assert_eq!(
            list_template_names(&catalog),
            vec!["web-server".to_string(), "cli".to_string()]
        );
        assert_eq!(list_addon_names(&catalog), vec!["github-setup".to_string()]);
    }

    #[test]
    fn filters_templates_by_category_tag() {
        let catalog = sample_catalog();
        assert_eq!(
            list_template_names_in_category(&catalog, "web"),
            vec!["web-server".to_string()]
        );
        assert!(list_template_names_in_category(&catalog, "unknown").is_empty());
    }

    #[test]
    fn formats_catalog_list_for_terminal() {
        let rendered = format_catalog_list("Templates", &["cli".to_string()]);
        assert_eq!(rendered, "Templates:\n  - cli");
    }

    #[test]
    fn loads_catalog_from_local_path() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("templates.json");
        let catalog = sample_catalog();
        fs::write(&path, serde_json::to_string(&catalog).expect("serialise"))
            .expect("write fixture");
        let loaded = load_catalog(Some(path.to_str().expect("utf8")), None).expect("load");
        assert_eq!(loaded, catalog);
    }

    #[test]
    fn reads_file_scheme_urls_from_disk() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("templates.json");
        let catalog = sample_catalog();
        fs::write(&path, serde_json::to_string(&catalog).expect("serialise"))
            .expect("write fixture");
        let url = format!("file://{}", path.to_str().expect("utf8"));
        let loaded = load_catalog(None, Some(&url)).expect("load");
        assert_eq!(loaded, catalog);
    }

    #[test]
    fn rejects_malformed_catalog_json() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("templates.json");
        fs::write(&path, "{ not json").expect("write fixture");
        let err = load_catalog(Some(path.to_str().expect("utf8")), None).expect_err("must fail");
        assert!(matches!(err, EngineError::CatalogParse(_)));
    }

    #[test]
    fn reports_missing_catalog_path() {
        let err =
            load_catalog(Some("/definitely/missing/templates.json"), None).expect_err("must fail");
        assert!(matches!(err, EngineError::CatalogRead { .. }));
    }

    #[test]
    fn resolves_fixture_path_from_explicit_dir() {
        let resolved = resolve_fixture_catalog_path(Some("/tmp/fixtures"));
        assert_eq!(
            resolved,
            PathBuf::from("/tmp/fixtures").join("templates.json")
        );
    }
}

#[cfg(test)]
mod extensions_key_tests {
    use super::*;

    #[test]
    fn reads_official_bank_extensions_key_as_addons() {
        let raw = r#"{
            "templates": [{"slug": "axum-starter", "url": "file:///bank/axum-starter"}],
            "extensions": [{"slug": "all-github-setup", "url": "file:///bank/all-github-setup"}]
        }"#;
        let catalog: Catalog = serde_json::from_str(raw).expect("parse");
        assert_eq!(catalog.templates.len(), 1);
        assert_eq!(catalog.addons.len(), 1);
        assert_eq!(catalog.addons[0].slug, "all-github-setup");
        assert_eq!(catalog.addons[0].url, "file:///bank/all-github-setup");
    }

    #[test]
    fn tolerates_entries_without_source_url() {
        let raw = r#"{"templates": [{"slug": "legacy"}], "addons": []}"#;
        let catalog: Catalog = serde_json::from_str(raw).expect("parse");
        assert_eq!(catalog.templates[0].url, String::new());
    }
}
