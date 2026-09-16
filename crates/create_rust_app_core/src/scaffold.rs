//! Project scaffolding: validate inputs and materialise a new project.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::catalog::Catalog;

/// Errors produced by the scaffolding engine.
#[derive(Debug, Error)]
pub enum EngineError {
    /// The project name is empty or contains unsupported characters.
    #[error("invalid project name '{name}': use letters, numbers, '-', '_'")]
    InvalidProjectName {
        /// The rejected project name.
        name: String,
    },

    /// A `--set key=value` override is malformed.
    #[error("invalid --set '{value}': expected key=value")]
    InvalidSetOverride {
        /// The rejected override.
        value: String,
    },

    /// The target directory is not empty and `--force` was not passed.
    #[error("target directory '{path}' is not empty (use --force to overwrite)")]
    TargetNotEmpty {
        /// The blocking directory.
        path: String,
    },

    /// The requested template slug is unknown to the catalog.
    #[error("unknown template '{slug}' (use --list-templates to browse the catalog)")]
    UnknownTemplate {
        /// The requested slug.
        slug: String,
    },

    /// A requested addon slug is unknown to the catalog.
    #[error("unknown addon '{slug}' (use --list-addons to browse the catalog)")]
    UnknownAddon {
        /// The requested slug.
        slug: String,
    },

    /// The catalog file could not be read.
    #[error("cannot read catalog at '{path}': {source}")]
    CatalogRead {
        /// Catalog location.
        path: String,
        /// Underlying I/O error.
        source: io::Error,
    },

    /// The catalog file is not valid JSON.
    #[error("cannot parse catalog: {0}")]
    CatalogParse(#[from] serde_json::Error),

    /// The remote catalog could not be fetched.
    #[error("cannot fetch catalog at '{url}': {source}")]
    CatalogFetch {
        /// Catalog URL.
        url: String,
        /// Underlying transport error (boxed: `ureq::Error` is large).
        #[source]
        source: Box<ureq::Error>,
    },

    /// The remote catalog body could not be read.
    #[error("cannot read catalog body at '{url}': {source}")]
    CatalogFetchBody {
        /// Catalog URL.
        url: String,
        /// Underlying I/O error.
        source: io::Error,
    },

    /// Project file I/O failed.
    #[error("cannot write project file '{path}': {source}")]
    ProjectWrite {
        /// Destination path.
        path: String,
        /// Underlying I/O error.
        source: io::Error,
    },
}

/// Options controlling one scaffold run.
#[derive(Debug, Clone)]
pub struct ScaffoldOptions {
    /// Project (directory) name, for example `my-api`.
    pub project: String,
    /// Template slug, URL, or `file://` path. Empty selects `default`.
    pub template: String,
    /// Addon slugs or URLs.
    pub addons: Vec<String>,
    /// Parsed `--set key=value` overrides.
    pub sets: Vec<(String, String)>,
    /// Allow non-empty target directories.
    pub force: bool,
    /// Skip network access; unknown slugs are recorded without validation.
    pub offline: bool,
    /// Keep the project directory when scaffolding fails.
    pub keep_on_failure: bool,
}

/// Check that `name` is a safe project path.
///
/// `name` may be a plain name (`my-app`) or a (relative or absolute) path
/// whose final segment is the package name; only that segment is validated.
pub fn validate_project_name(name: &str) -> Result<(), EngineError> {
    let base = Path::new(name)
        .file_name()
        .and_then(|segment| segment.to_str())
        .unwrap_or("");
    let valid = !base.is_empty()
        && base
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_');
    if valid {
        Ok(())
    } else {
        Err(EngineError::InvalidProjectName {
            name: name.to_string(),
        })
    }
}

/// Parse one `--set key=value` override.
pub fn parse_set_override(raw: &str) -> Result<(String, String), EngineError> {
    match raw.split_once('=') {
        Some(("", _)) => Err(EngineError::InvalidSetOverride {
            value: raw.to_string(),
        }),
        Some((key, value)) => Ok((key.to_string(), value.to_string())),
        None => Err(EngineError::InvalidSetOverride {
            value: raw.to_string(),
        }),
    }
}

fn is_external_spec(spec: &str) -> bool {
    spec.contains("://") || spec.contains('/')
}

fn resolve_template<'a>(
    template: &str,
    catalog: &'a Catalog,
    offline: bool,
) -> Result<Option<&'a crate::catalog::TemplateEntry>, EngineError> {
    if template.is_empty() || template == "default" || is_external_spec(template) {
        return Ok(None);
    }
    if offline {
        return Ok(None);
    }
    catalog
        .templates
        .iter()
        .find(|entry| entry.slug == *template)
        .map(Some)
        .ok_or_else(|| EngineError::UnknownTemplate {
            slug: template.to_string(),
        })
}

fn resolve_addons(catalog: &Catalog, addons: &[String], offline: bool) -> Result<(), EngineError> {
    for addon in addons {
        if is_external_spec(addon) || offline {
            continue;
        }
        if !catalog.addons.iter().any(|entry| entry.slug == **addon) {
            return Err(EngineError::UnknownAddon {
                slug: addon.clone(),
            });
        }
    }
    Ok(())
}

fn target_is_empty(dir: &Path) -> bool {
    match fs::read_dir(dir) {
        Ok(mut entries) => entries.next().is_none(),
        Err(_) => true,
    }
}

fn write_file(path: &Path, contents: &str) -> Result<(), EngineError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| EngineError::ProjectWrite {
            path: parent.to_string_lossy().to_string(),
            source,
        })?;
    }
    fs::write(path, contents).map_err(|source| EngineError::ProjectWrite {
        path: path.to_string_lossy().to_string(),
        source,
    })
}

fn render_cargo_toml(project: &str, template: &str) -> String {
    format!(
        r#"[package]
name = "{project}"
version = "0.1.0"
edition = "2021"
description = "Scaffolded with Create Rust App from template '{template}'"

[dependencies]
"#
    )
}

fn render_main_rs(project: &str) -> String {
    format!(
        r#"fn main() {{
    println!("Hello from {project}!");
}}
"#
    )
}

fn render_config(
    project: &str,
    template: &str,
    addons: &[String],
    sets: &[(String, String)],
) -> String {
    let config = serde_json::json!({
        "project": project,
        "template": template,
        "addons": addons,
        "set": sets
            .iter()
            .map(|(key, value)| serde_json::json!({"key": key, "value": value}))
            .collect::<Vec<_>>(),
    });
    serde_json::to_string_pretty(&config).expect("config is serialisable")
}

/// Scaffold a new project directory and return its path.
pub fn scaffold(options: &ScaffoldOptions, catalog: &Catalog) -> Result<PathBuf, EngineError> {
    validate_project_name(&options.project)?;
    let template = if options.template.is_empty() {
        "default".to_string()
    } else {
        options.template.clone()
    };
    let _resolved = resolve_template(&template, catalog, options.offline)?;
    resolve_addons(catalog, &options.addons, options.offline)?;

    let target = PathBuf::from(&options.project);
    if target.exists() && !target_is_empty(&target) && !options.force {
        return Err(EngineError::TargetNotEmpty {
            path: options.project.clone(),
        });
    }
    let package = Path::new(&options.project)
        .file_name()
        .and_then(|segment| segment.to_str())
        .unwrap_or(&options.project)
        .to_string();

    let result = (|| -> Result<PathBuf, EngineError> {
        write_file(
            &target.join("Cargo.toml"),
            &render_cargo_toml(&package, &template),
        )?;
        write_file(&target.join("src/main.rs"), &render_main_rs(&package))?;
        write_file(
            &target.join("cra.config.json"),
            &render_config(&options.project, &template, &options.addons, &options.sets),
        )?;
        Ok(target.clone())
    })();

    if result.is_err() && !options.keep_on_failure && target.exists() {
        let _ = fs::remove_dir_all(&target);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{AddonEntry, TemplateEntry};

    fn sample_catalog() -> Catalog {
        Catalog {
            templates: vec![TemplateEntry {
                slug: "web-server".to_string(),
                description: "Axum web server".to_string(),
                tags: vec!["web".to_string()],
            }],
            addons: vec![AddonEntry {
                slug: "github-setup".to_string(),
                description: "CI workflows".to_string(),
            }],
        }
    }

    fn options_in(dir: &Path, project: &str) -> ScaffoldOptions {
        ScaffoldOptions {
            project: dir.join(project).to_string_lossy().to_string(),
            template: "web-server".to_string(),
            addons: vec!["github-setup".to_string()],
            sets: vec![("key".to_string(), "value".to_string())],
            force: false,
            offline: false,
            keep_on_failure: false,
        }
    }

    #[test]
    fn accepts_valid_project_names() {
        for name in ["my-app", "my_app", "app123", "my/app", "/tmp/x/my-app"] {
            validate_project_name(name).expect("valid name");
        }
    }

    #[test]
    fn rejects_invalid_project_names() {
        for name in ["", "my app", "café", "/", "my/bad name"] {
            validate_project_name(name).expect_err("invalid name");
        }
    }

    #[test]
    fn parses_set_overrides() {
        assert_eq!(
            parse_set_override("key=value").expect("parsed"),
            ("key".to_string(), "value".to_string())
        );
        assert_eq!(
            parse_set_override("key=a=b").expect("parsed"),
            ("key".to_string(), "a=b".to_string())
        );
        parse_set_override("novalue").expect_err("must fail");
        parse_set_override("=value").expect_err("must fail");
    }

    #[test]
    fn scaffolds_expected_files() {
        let dir = tempfile::tempdir().expect("tempdir");
        let options = options_in(dir.path(), "my-api");
        let target = scaffold(&options, &sample_catalog()).expect("scaffold");
        assert!(target.join("Cargo.toml").exists());
        assert!(target.join("src/main.rs").exists());
        assert!(target.join("cra.config.json").exists());
        let manifest = fs::read_to_string(target.join("Cargo.toml")).expect("read manifest");
        assert!(manifest.contains("web-server"));
    }

    #[test]
    fn refuses_non_empty_target_without_force() {
        let dir = tempfile::tempdir().expect("tempdir");
        let project_dir = dir.path().join("taken");
        fs::create_dir_all(&project_dir).expect("mkdir");
        fs::write(project_dir.join("existing.txt"), "x").expect("write");
        let mut options = options_in(dir.path(), "taken");
        let err = scaffold(&options, &sample_catalog()).expect_err("must fail");
        assert!(matches!(err, EngineError::TargetNotEmpty { .. }));
        options.force = true;
        scaffold(&options, &sample_catalog()).expect("force succeeds");
    }

    #[test]
    fn rejects_unknown_template_and_addon() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut options = options_in(dir.path(), "proj");
        options.template = "nope".to_string();
        let err = scaffold(&options, &sample_catalog()).expect_err("must fail");
        assert!(matches!(err, EngineError::UnknownTemplate { .. }));
        options.template = "web-server".to_string();
        options.addons = vec!["nope".to_string()];
        let err = scaffold(&options, &sample_catalog()).expect_err("must fail");
        assert!(matches!(err, EngineError::UnknownAddon { .. }));
    }

    #[test]
    fn offline_mode_records_unknown_slugs() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut options = options_in(dir.path(), "proj");
        options.template = "future-template".to_string();
        options.addons = vec!["future-addon".to_string()];
        options.offline = true;
        scaffold(&options, &sample_catalog()).expect("offline records slugs");
    }
}
