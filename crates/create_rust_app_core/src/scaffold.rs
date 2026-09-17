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

    /// The catalog contains no templates, so no default can be selected.
    #[error("the catalog contains no templates (pass --template explicitly)")]
    EmptyCatalog,

    /// A catalog entry has no source URL to materialise from.
    #[error("template '{slug}' has no source URL in the catalog")]
    TemplateHasNoSource {
        /// The entry slug.
        slug: String,
    },

    /// A catalog addon has no source URL to materialise from.
    #[error("addon '{slug}' has no source URL in the catalog")]
    AddonHasNoSource {
        /// The entry slug.
        slug: String,
    },

    /// A template or addon source could not be fetched.
    #[error("cannot fetch template source '{request}': {detail}")]
    TemplateFetch {
        /// Requested source (slug, URL, or path).
        request: String,
        /// Human-readable cause.
        detail: String,
    },

    /// Fetched template content could not be materialised.
    #[error("cannot materialise template into '{path}': {detail}")]
    TemplateMaterialize {
        /// Destination path.
        path: String,
        /// Human-readable cause.
        detail: String,
    },

    /// Offline mode with a cold cache.
    #[error(
        "offline mode: no cached copy of '{request}' (scaffold once online to populate the cache)"
    )]
    OfflineCacheMiss {
        /// Requested source (slug or URL).
        request: String,
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
    /// Skip network access; remote sources resolve from the local cache.
    pub offline: bool,
    /// Keep the project directory when scaffolding fails.
    pub keep_on_failure: bool,
    /// Cache root override (`None` falls back to `CRA_CACHE_DIR`, then the
    /// platform default from [`crate::cache::cache_dir`]).
    pub cache_dir: Option<PathBuf>,
    /// Skip cache reads and writes; remote sources are always fetched fresh.
    /// Honours the `CRA_NO_CACHE` environment variable when set to `1`.
    pub no_cache: bool,
    /// Git ref (branch or tag) to check out when cloning remote sources.
    pub pin: Option<String>,
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
    spec.contains("://") || spec.contains('/') || Path::new(spec).is_dir()
}

fn strip_file_scheme(location: &str) -> Option<&str> {
    location.strip_prefix("file://")
}

/// A template or addon source resolved to a fetchable location.
enum ResolvedSource {
    /// Local directory, used directly (works offline).
    Local(PathBuf),
    /// Remote git source with an optional subdirectory.
    Remote {
        /// Repository URL without query or fragment.
        url: String,
        /// Subdirectory inside the repository, if any.
        subdir: Option<String>,
    },
}

fn sanitize_cache_key(raw: &str) -> String {
    raw.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn cache_key_for_url(url: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    url.hash(&mut hasher);
    format!("url-{:016x}", hasher.finish())
}

/// Split a `?subdir=` (or `&subdir=`) query off a repository URL.
fn split_subdir(url: &str) -> (String, Option<String>) {
    let marker = url
        .find("?subdir=")
        .map(|index| (index, index))
        .or_else(|| url.find("&subdir=").map(|index| (index, index)));
    match marker {
        Some((cut, value_at)) => {
            let base = url[..cut].to_string();
            let mut subdir = url[value_at + 8..].to_string();
            if let Some(hash) = subdir.find('#') {
                subdir.truncate(hash);
            }
            let subdir = subdir.trim_matches('/').to_string();
            (
                base,
                if subdir.is_empty() {
                    None
                } else {
                    Some(subdir)
                },
            )
        }
        None => (url.to_string(), None),
    }
}

fn resolve_source(spec: &str, entry_url: Option<&str>) -> Result<ResolvedSource, EngineError> {
    if let Some(url) = entry_url {
        return resolve_source_url(url, spec);
    }
    resolve_source_url(spec, spec)
}

fn resolve_source_url(url: &str, display: &str) -> Result<ResolvedSource, EngineError> {
    if let Some(local) = strip_file_scheme(url) {
        let dir = PathBuf::from(local);
        if dir.is_dir() {
            return Ok(ResolvedSource::Local(dir));
        }
        return Err(EngineError::TemplateFetch {
            request: display.to_string(),
            detail: format!("local directory '{}' does not exist", dir.display()),
        });
    }
    if !url.contains("://") {
        let dir = PathBuf::from(url);
        if dir.is_dir() {
            return Ok(ResolvedSource::Local(dir));
        }
    }
    let (base, subdir) = split_subdir(url);
    Ok(ResolvedSource::Remote { url: base, subdir })
}

fn resolve_template_source(
    template: &str,
    catalog: &Catalog,
) -> Result<(ResolvedSource, String), EngineError> {
    if !is_external_spec(template) {
        let entry = catalog
            .templates
            .iter()
            .find(|entry| entry.slug == *template)
            .ok_or_else(|| EngineError::UnknownTemplate {
                slug: template.to_string(),
            })?;
        if entry.url.is_empty() {
            return Err(EngineError::TemplateHasNoSource {
                slug: template.to_string(),
            });
        }
        let key = format!("template-{}", sanitize_cache_key(template));
        return resolve_source(template, Some(&entry.url)).map(|source| (source, key));
    }
    resolve_source(template, None)
        .map(|source| (source, format!("template-{}", cache_key_for_url(template))))
}

fn resolve_addon_source(
    addon: &str,
    catalog: &Catalog,
) -> Result<(ResolvedSource, String), EngineError> {
    if !is_external_spec(addon) {
        let entry = catalog
            .addons
            .iter()
            .find(|entry| entry.slug == *addon)
            .ok_or_else(|| EngineError::UnknownAddon {
                slug: addon.to_string(),
            })?;
        if entry.url.is_empty() {
            return Err(EngineError::AddonHasNoSource {
                slug: addon.to_string(),
            });
        }
        let key = format!("addon-{}", sanitize_cache_key(addon));
        return resolve_source(addon, Some(&entry.url)).map(|source| (source, key));
    }
    resolve_source(addon, None)
        .map(|source| (source, format!("addon-{}", cache_key_for_url(addon))))
}

fn cache_root(options: &ScaffoldOptions) -> PathBuf {
    if let Some(dir) = &options.cache_dir {
        if !dir.as_os_str().is_empty() {
            return dir.clone();
        }
    }
    crate::cache::cache_dir()
}

fn cache_bypassed(options: &ScaffoldOptions) -> bool {
    options.no_cache
        || std::env::var("CRA_NO_CACHE").as_deref() == Ok("1")
        || std::env::var("CRA_REFRESH").as_deref() == Ok("always")
}

fn refresh_manual(options: &ScaffoldOptions) -> bool {
    options.offline || std::env::var("CRA_REFRESH").as_deref() == Ok("manual")
}

fn run_git(args: &[&str], cwd: &Path, source: &str) -> Result<String, EngineError> {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|source_error| EngineError::TemplateFetch {
            request: source.to_string(),
            detail: if source_error.kind() == io::ErrorKind::NotFound {
                "git executable not found in PATH (required to fetch remote templates)".to_string()
            } else {
                format!("cannot run git: {source_error}")
            },
        })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let detail = stderr.lines().last().unwrap_or("git command failed").trim();
        return Err(EngineError::TemplateFetch {
            request: source.to_string(),
            detail: format!("git {} failed: {detail}", args.join(" ")),
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Fetch a remote source and return the materialisable directory.
///
/// Returns the usable directory plus, for cache-bypassed clones, the
/// scratch root the caller must remove with [`cleanup_scratch`] once
/// content has been copied out of it.
fn fetch_remote(
    key: &str,
    url: &str,
    subdir: Option<&str>,
    display: &str,
    options: &ScaffoldOptions,
) -> Result<(PathBuf, Option<PathBuf>), EngineError> {
    let cached = cache_root(options).join("banks").join(key);
    let want_subdir = subdir.map(str::to_string);
    let cached_subdir = match &want_subdir {
        Some(sub) => cached.join(sub),
        None => cached.clone(),
    };
    if refresh_manual(options) || cache_bypassed(options) && options.offline {
        if cached_subdir.is_dir() {
            return Ok((cached_subdir, None));
        }
        return Err(EngineError::OfflineCacheMiss {
            request: display.to_string(),
        });
    }
    if cache_bypassed(options) {
        let tmp = scratch_dir(display)?;
        let result = clone_sparse(url, subdir, options.pin.as_deref(), &tmp, display);
        // The scratch dir outlives this call: the caller copies content out
        // of it and then removes it via `cleanup_scratch`.
        return result.map(|()| {
            let usable = match &want_subdir {
                Some(sub) => tmp.join(sub),
                None => tmp.clone(),
            };
            (usable, Some(tmp))
        });
    }
    if cached.exists() {
        let _ = fs::remove_dir_all(&cached);
    }
    if let Some(parent) = cached.parent() {
        fs::create_dir_all(parent).map_err(|source| EngineError::TemplateFetch {
            request: display.to_string(),
            detail: format!("cannot create cache directory: {source}"),
        })?;
    }
    clone_sparse(url, subdir, options.pin.as_deref(), &cached, display)?;
    if cached_subdir.is_dir() {
        Ok((cached_subdir, None))
    } else {
        Err(EngineError::TemplateFetch {
            request: display.to_string(),
            detail: format!(
                "subdirectory '{}' not found in {url}",
                want_subdir.unwrap_or_default()
            ),
        })
    }
}

/// Best-effort removal of a scratch clone directory.
///
/// A cleanup failure never fails the scaffold; it only leaves a temp dir
/// behind for the OS to reclaim.
fn cleanup_scratch(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

fn scratch_dir(display: &str) -> Result<PathBuf, EngineError> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("create-rust-app-{}-{id}", std::process::id()));
    fs::create_dir_all(&dir).map_err(|source| EngineError::TemplateFetch {
        request: display.to_string(),
        detail: format!("cannot create scratch directory: {source}"),
    })?;
    Ok(dir)
}

fn clone_sparse(
    url: &str,
    subdir: Option<&str>,
    pin: Option<&str>,
    dest: &Path,
    display: &str,
) -> Result<(), EngineError> {
    if let Some(parent) = dest.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|source| EngineError::TemplateFetch {
                request: display.to_string(),
                detail: format!("cannot create destination directory: {source}"),
            })?;
        }
    }
    let mut clone_args: Vec<&str> = vec![
        "clone",
        "--quiet",
        "--depth",
        "1",
        "--filter=blob:none",
        "--sparse",
    ];
    if let Some(git_ref) = pin {
        if !git_ref.is_empty() {
            clone_args.push("--branch");
            clone_args.push(git_ref);
        }
    }
    let url_owned = url.to_string();
    clone_args.push(&url_owned);
    let dest_owned = dest.to_string_lossy().into_owned();
    clone_args.push(&dest_owned);
    run_git(&clone_args, Path::new("."), display)?;
    if let Some(sub) = subdir {
        run_git(&["sparse-checkout", "set", sub], dest, display)?;
    }
    Ok(())
}

fn copy_tree(src: &Path, dst: &Path, display: &str) -> Result<(), EngineError> {
    let entries = fs::read_dir(src).map_err(|source| EngineError::TemplateFetch {
        request: display.to_string(),
        detail: format!("cannot read template source '{}': {source}", src.display()),
    })?;
    let mut deferred: Vec<(PathBuf, PathBuf)> = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| EngineError::TemplateFetch {
            request: display.to_string(),
            detail: format!("cannot list template source: {source}"),
        })?;
        if entry.file_name() == ".git" {
            continue;
        }
        let from = entry.path();
        let to = dst.join(entry.file_name());
        let file_type = entry
            .file_type()
            .map_err(|source| EngineError::TemplateFetch {
                request: display.to_string(),
                detail: format!("cannot stat '{}': {source}", from.display()),
            })?;
        if file_type.is_dir() {
            fs::create_dir_all(&to).map_err(|source| EngineError::ProjectWrite {
                path: to.to_string_lossy().to_string(),
                source,
            })?;
            copy_tree(&from, &to, display)?;
        } else if file_type.is_file() {
            if let Some(parent) = to.parent() {
                fs::create_dir_all(parent).map_err(|source| EngineError::ProjectWrite {
                    path: parent.to_string_lossy().to_string(),
                    source,
                })?;
            }
            // Fragments apply after every plain copy in this overlay pass,
            // so readdir order can never clobber an applied fragment with
            // the file it extends.
            if strip_append_suffix(&from).is_some() {
                deferred.push((from, to));
                continue;
            }
            if entry.file_name() == "Cargo.toml" && to.is_file() {
                merge_manifest(&to, &from, display)?;
            } else {
                fs::copy(&from, &to).map_err(|source| EngineError::ProjectWrite {
                    path: to.to_string_lossy().to_string(),
                    source,
                })?;
            }
        }
    }
    for (from, to) in deferred {
        let stripped = strip_append_suffix(&from).expect("append suffix");
        let target = to.parent().unwrap_or_else(|| Path::new("")).join(stripped);
        append_fragment(&from, &target, display)?;
    }
    Ok(())
}

/// Dependency tables merged from overlay manifests into the project manifest.
///
/// Only these tables plus [`MERGED_TARGET_TABLES`] are merged; every other
/// overlay section is ignored by contract (extension manifests must not
/// restate `[package]` or other metadata).
const MERGED_DEP_TABLES: [&str; 3] = ["dependencies", "dev-dependencies", "build-dependencies"];

/// Target sections merged from overlay manifests into the project manifest.
///
/// Entries merge by `name`: missing entries are added, identical entries are
/// skipped, and a conflicting declaration for the same target fails the
/// scaffold naming both specs.
const MERGED_TARGET_TABLES: [&str; 3] = ["bench", "example", "test"];

/// Strip a trailing `.append` overlay suffix, returning the target file name.
///
/// `router.rs.append` merges into `router.rs`; plain files return `None`.
fn strip_append_suffix(path: &Path) -> Option<String> {
    let name = path.file_name()?.to_str()?;
    name.strip_suffix(".append").map(str::to_string)
}

/// Append an overlay fragment to a project file.
///
/// A missing target is created from the fragment; otherwise the fragment is
/// appended after a single trailing newline. This is how extensions register
/// routers, modules, env keys, and docs without forking template files.
fn append_fragment(fragment: &Path, target: &Path, display: &str) -> Result<(), EngineError> {
    let addition = fs::read(fragment).map_err(|source| EngineError::TemplateFetch {
        request: display.to_string(),
        detail: format!(
            "cannot read overlay fragment '{}': {source}",
            fragment.display()
        ),
    })?;
    if addition.is_empty() {
        return Ok(());
    }
    let mut base = if target.is_file() {
        fs::read(target).map_err(|source| EngineError::ProjectWrite {
            path: target.to_string_lossy().to_string(),
            source,
        })?
    } else {
        Vec::new()
    };
    if !base.is_empty() && !base.ends_with(b"\n") {
        base.push(b'\n');
    }
    base.extend_from_slice(&addition);
    fs::write(target, base).map_err(|source| EngineError::ProjectWrite {
        path: target.to_string_lossy().to_string(),
        source,
    })
}

/// Merge an overlay `Cargo.toml` into the project manifest.
///
/// Dependency entries missing from the project are added; identical entries
/// are skipped. A conflicting requirement for the same dependency fails the
/// scaffold with both specs named, so extension authors resolve version
/// clashes explicitly instead of shipping silent downgrades.
fn merge_manifest(target: &Path, overlay: &Path, display: &str) -> Result<(), EngineError> {
    let target_raw = fs::read_to_string(target).map_err(|source| EngineError::ProjectWrite {
        path: target.to_string_lossy().to_string(),
        source,
    })?;
    let overlay_raw = fs::read_to_string(overlay).map_err(|source| EngineError::TemplateFetch {
        request: display.to_string(),
        detail: format!(
            "cannot read overlay manifest '{}': {source}",
            overlay.display()
        ),
    })?;
    let mut document: toml_edit::DocumentMut =
        target_raw
            .parse()
            .map_err(|source| EngineError::TemplateMaterialize {
                path: target.to_string_lossy().to_string(),
                detail: format!("cannot parse project Cargo.toml: {source}"),
            })?;
    let overlay_doc: toml_edit::DocumentMut =
        overlay_raw
            .parse()
            .map_err(|source| EngineError::TemplateMaterialize {
                path: overlay.to_string_lossy().to_string(),
                detail: format!("cannot parse overlay Cargo.toml: {source}"),
            })?;
    for table in MERGED_DEP_TABLES {
        let Some(items) = overlay_doc.get(table).and_then(|node| node.as_table()) else {
            continue;
        };
        let target_table =
            document[table].or_insert(toml_edit::Item::Table(toml_edit::Table::new()));
        let Some(target_items) = target_table.as_table_mut() else {
            return Err(EngineError::TemplateMaterialize {
                path: target.to_string_lossy().to_string(),
                detail: format!("project Cargo.toml [{table}] is not a table"),
            });
        };
        for (name, item) in items {
            if target_items.get(name).is_none() {
                target_items.insert(name, item.clone());
                continue;
            }
            let existing = target_items.get(name).expect("dependency present").clone();
            if existing.to_string() == item.to_string() {
                continue;
            }
            merge_dep_spec(target_items, name, &existing, item, target, overlay)?;
        }
    }
    for table in MERGED_TARGET_TABLES {
        merge_target_section(&mut document, &overlay_doc, table, target, overlay)?;
    }
    fs::write(target, document.to_string()).map_err(|source| EngineError::ProjectWrite {
        path: target.to_string_lossy().to_string(),
        source,
    })
}

/// Merge one `[[bench]]`/`[[example]]`/`[[test]]` section by target `name`.
fn merge_target_section(
    document: &mut toml_edit::DocumentMut,
    overlay_doc: &toml_edit::DocumentMut,
    table: &str,
    target: &Path,
    overlay: &Path,
) -> Result<(), EngineError> {
    let Some(items) = overlay_doc
        .get(table)
        .and_then(|node| node.as_array_of_tables())
    else {
        return Ok(());
    };
    if items.is_empty() {
        return Ok(());
    }
    if document.get(table).is_none() {
        document[table] = toml_edit::Item::ArrayOfTables(toml_edit::ArrayOfTables::new());
    }
    let Some(target_items) = document
        .get_mut(table)
        .and_then(|node| node.as_array_of_tables_mut())
    else {
        return Err(EngineError::TemplateMaterialize {
            path: target.to_string_lossy().to_string(),
            detail: format!("project Cargo.toml [[{table}]] is not an array of tables"),
        });
    };
    for item in items {
        let name = item
            .get("name")
            .and_then(|node| node.as_str())
            .unwrap_or("");
        let position = target_items.iter().position(|existing| {
            existing
                .get("name")
                .and_then(|node| node.as_str())
                .unwrap_or("")
                == name
        });
        let Some(position) = position else {
            target_items.push(item.clone());
            continue;
        };
        if target_items
            .get(position)
            .expect("target present")
            .to_string()
            == item.to_string()
        {
            continue;
        }
        return Err(EngineError::TemplateMaterialize {
            path: target.to_string_lossy().to_string(),
            detail: format!(
                "conflicting [[{table}]] {name:?} in overlay '{}'",
                overlay.display()
            ),
        });
    }
    Ok(())
}

/// Split a dependency spec into its version requirement, feature list, and
/// remaining keys (everything except `version` and `features`, rendered for
/// comparison). Exotic specs (dotted table headers) are fingerprinted by
/// their rendered text so differing ones always conflict.
fn dep_parts(item: &toml_edit::Item) -> (Option<String>, Vec<String>, Vec<(String, String)>) {
    let table = match item {
        toml_edit::Item::Value(toml_edit::Value::String(text)) => {
            return (Some(text.value().to_string()), Vec::new(), Vec::new());
        }
        toml_edit::Item::Value(toml_edit::Value::InlineTable(table)) => table,
        _ => {
            return (
                None,
                Vec::new(),
                vec![("spec".to_string(), item.to_string())],
            );
        }
    };
    let version = table
        .get("version")
        .and_then(toml_edit::Value::as_str)
        .map(str::to_string);
    let features = table
        .get("features")
        .and_then(toml_edit::Value::as_array)
        .map(|array| {
            array
                .iter()
                .filter_map(toml_edit::Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    let mut rest: Vec<(String, String)> = table
        .iter()
        .filter(|(key, _)| *key != "version" && *key != "features")
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect();
    rest.sort();
    (version, features, rest)
}

/// Merge one conflicting dependency spec from an overlay manifest.
///
/// Additive differences compose: when both sides require the same version
/// with the same remaining keys, their `features` unite in the project
/// manifest (this is how extensions enable extra functionality on template
/// dependencies, e.g. `tower-http` with `cors` next to `trace`). Anything
/// else — different versions, different sources or flags — fails naming
/// both specs so authors resolve the clash explicitly.
#[allow(clippy::too_many_arguments)]
fn merge_dep_spec(
    target_items: &mut toml_edit::Table,
    name: &str,
    existing: &toml_edit::Item,
    item: &toml_edit::Item,
    target: &Path,
    overlay: &Path,
) -> Result<(), EngineError> {
    let conflict = || {
        EngineError::TemplateMaterialize {
        path: target.to_string_lossy().to_string(),
        detail: format!(
            "dependency conflict on '{name}': project requires {existing} but overlay '{}' requires {item}",
            overlay.display(),
        ),
    }
    };
    let (existing_version, existing_features, existing_rest) = dep_parts(existing);
    let (overlay_version, overlay_features, overlay_rest) = dep_parts(item);
    if existing_version != overlay_version || existing_rest != overlay_rest {
        return Err(conflict());
    }
    let mut merged = existing_features;
    for feature in overlay_features {
        if !merged.contains(&feature) {
            merged.push(feature);
        }
    }
    if merged == dep_parts(existing).1 {
        return Ok(());
    }
    match target_items.get_mut(name) {
        Some(toml_edit::Item::Value(toml_edit::Value::InlineTable(table))) => {
            let mut array = toml_edit::Array::new();
            for feature in &merged {
                array.push(toml_edit::Value::from(feature.as_str()));
            }
            table.insert("features", toml_edit::Value::Array(array));
        }
        // The project pins a bare version string while the overlay carries
        // the fuller spec (features): adopt the overlay entry.
        _ => {
            target_items.insert(name, item.clone());
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ManifestSection {
    Other,
    Package,
    Bin,
    Lib,
}

/// Rename the scaffolded crate in `Cargo.toml`.
///
/// Only `name = "..."` lines inside `[package]`, `[[bin]]`, and `[lib]`
/// sections are rewritten (kebab-case for package/bin targets,
/// snake_case for the library target); everything else is preserved byte
/// for byte so formatting and comments survive.
fn rename_package(manifest: &Path, kebab: &str, snake: &str) -> io::Result<()> {
    let raw = fs::read_to_string(manifest)?;
    let mut section = ManifestSection::Other;
    let mut out = String::with_capacity(raw.len());
    for line in raw.split_inclusive('\n') {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            section = match trimmed {
                "[package]" => ManifestSection::Package,
                "[[bin]]" => ManifestSection::Bin,
                "[lib]" => ManifestSection::Lib,
                _ => ManifestSection::Other,
            };
            out.push_str(line);
            continue;
        }
        let is_name_line = trimmed.starts_with("name")
            && trimmed[4..].trim_start().starts_with('=')
            && matches!(
                section,
                ManifestSection::Package | ManifestSection::Bin | ManifestSection::Lib
            );
        if !is_name_line {
            out.push_str(line);
            continue;
        }
        let replacement = match section {
            ManifestSection::Lib => snake,
            _ => kebab,
        };
        let bytes = line.as_bytes();
        if let (Some(first), Some(last)) = (
            bytes.iter().position(|b| *b == b'"'),
            bytes.iter().rposition(|b| *b == b'"'),
        ) {
            if first < last {
                out.push_str(&line[..=first]);
                out.push_str(replacement);
                out.push_str(&line[last..]);
                continue;
            }
        }
        out.push_str(line);
    }
    fs::write(manifest, out)
}

/// Merge the engine-written project record into the scaffolded
/// `cra.config.json`, preserving template-declared keys such as
/// `customOptions`.
fn merge_project_config(
    dir: &Path,
    project: &str,
    template: &str,
    addons: &[String],
    sets: &[(String, String)],
) -> Result<(), EngineError> {
    let path = dir.join("cra.config.json");
    let mut value = match fs::read_to_string(&path) {
        Ok(raw) => serde_json::from_str(&raw).unwrap_or(serde_json::json!({})),
        Err(_) => serde_json::json!({}),
    };
    let object = value
        .as_object_mut()
        .ok_or_else(|| EngineError::TemplateMaterialize {
            path: path.to_string_lossy().to_string(),
            detail: "cra.config.json is not a JSON object".to_string(),
        })?;
    object.insert(
        "project".to_string(),
        serde_json::Value::String(project.to_string()),
    );
    object.insert(
        "template".to_string(),
        serde_json::Value::String(template.to_string()),
    );
    object.insert(
        "addons".to_string(),
        serde_json::Value::Array(
            addons
                .iter()
                .map(|addon| serde_json::Value::String(addon.clone()))
                .collect(),
        ),
    );
    object.insert(
        "set".to_string(),
        serde_json::Value::Array(
            sets.iter()
                .map(|(key, value)| serde_json::json!({"key": key, "value": value}))
                .collect(),
        ),
    );
    let rendered = serde_json::to_string_pretty(&value).map_err(|source| {
        EngineError::TemplateMaterialize {
            path: path.to_string_lossy().to_string(),
            detail: format!("cannot serialise cra.config.json: {source}"),
        }
    })?;
    write_file(&path, &format!("{rendered}\n"))
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

fn snake_case(name: &str) -> String {
    name.replace('-', "_")
}

/// Package, library, and binary target names declared in a `Cargo.toml`.
#[derive(Debug, Default)]
struct ManifestNames {
    package: Option<String>,
    lib: Option<String>,
    bins: Vec<String>,
}

/// Read the `[package]` name plus `[lib]`/`[[bin]]` target names from a manifest.
fn read_manifest_names(manifest: &Path) -> io::Result<ManifestNames> {
    let raw = fs::read_to_string(manifest)?;
    let mut names = ManifestNames::default();
    let mut section = ManifestSection::Other;
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            section = match trimmed {
                "[package]" => ManifestSection::Package,
                "[[bin]]" => ManifestSection::Bin,
                "[lib]" => ManifestSection::Lib,
                _ => ManifestSection::Other,
            };
            continue;
        }
        if !(trimmed.starts_with("name") && trimmed[4..].trim_start().starts_with('=')) {
            continue;
        }
        let Some(value) = quoted_value(line) else {
            continue;
        };
        match section {
            ManifestSection::Package => names.package = Some(value),
            ManifestSection::Lib => names.lib = Some(value),
            ManifestSection::Bin => names.bins.push(value),
            ManifestSection::Other => {}
        }
    }
    Ok(names)
}

/// Extract the first double-quoted value from a manifest line.
fn quoted_value(line: &str) -> Option<String> {
    let bytes = line.as_bytes();
    let (first, last) = (
        bytes.iter().position(|b| *b == b'"')?,
        bytes.iter().rposition(|b| *b == b'"')?,
    );
    if first < last {
        Some(line[first + 1..last].to_string())
    } else {
        None
    }
}

/// Replace `from` with `to` only at identifier boundaries: characters around
/// the match must not be ASCII alphanumerics, `_`, or any of `extra` (used
/// for `-` so kebab-case names never rewrite a longer hyphenated name).
fn replace_bounded(text: &str, from: &str, to: &str, extra: &[char]) -> String {
    if from.is_empty() || from == to {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(index) = rest.find(from) {
        let (before, after) = rest.split_at(index);
        let after_match = &after[from.len()..];
        let before_ok = before.chars().next_back().map_or(true, |c| {
            !(c.is_ascii_alphanumeric() || c == '_' || extra.contains(&c))
        });
        let after_ok = after_match.chars().next().map_or(true, |c| {
            !(c.is_ascii_alphanumeric() || c == '_' || extra.contains(&c))
        });
        out.push_str(before);
        if before_ok && after_ok {
            out.push_str(to);
        } else {
            out.push_str(from);
        }
        rest = after_match;
    }
    out.push_str(rest);
    out
}

/// Rewrite references to the template's original package/library names across
/// the scaffolded project (Rust sources, `Cargo.lock`, docs), so the renamed
/// project compiles without manual edits. Binary files are left untouched.
fn rewrite_package_references(
    dir: &Path,
    old: &ManifestNames,
    package: &str,
    snake: &str,
) -> io::Result<()> {
    let mut renames: Vec<(String, String, Vec<char>)> = Vec::new();
    if let Some(old_package) = &old.package {
        renames.push((old_package.clone(), package.to_string(), vec!['-']));
    }
    if let Some(old_lib) = &old.lib {
        renames.push((old_lib.clone(), snake.to_string(), vec![]));
    }
    for old_bin in &old.bins {
        renames.push((old_bin.clone(), package.to_string(), vec!['-']));
    }
    renames.retain(|(from, to, _)| !from.is_empty() && from != to);
    if renames.is_empty() {
        return Ok(());
    }
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        for entry in fs::read_dir(&current)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let path = entry.path();
            if file_type.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name == ".git" || name == "target" {
                        continue;
                    }
                }
                stack.push(path);
                continue;
            }
            if !file_type.is_file() {
                continue;
            }
            let bytes = fs::read(&path)?;
            if bytes.contains(&0) {
                continue;
            }
            let Ok(text) = String::from_utf8(bytes) else {
                continue;
            };
            let mut updated = text.clone();
            for (from, to, extra) in &renames {
                updated = replace_bounded(&updated, from, to, extra);
            }
            if updated != text {
                fs::write(&path, updated)?;
            }
        }
    }
    Ok(())
}

/// Resolve a template or addon spec to a materialisable directory.
///
/// Returns the directory plus, for cache-bypassed clones, the scratch
/// root the caller must remove with [`cleanup_scratch`] once content has
/// been copied out of it.
fn materialise_source(
    source: &ResolvedSource,
    cache_key: &str,
    display: &str,
    options: &ScaffoldOptions,
) -> Result<(PathBuf, Option<PathBuf>), EngineError> {
    match source {
        ResolvedSource::Local(dir) => Ok((dir.clone(), None)),
        ResolvedSource::Remote { url, subdir } => {
            fetch_remote(cache_key, url, subdir.as_deref(), display, options)
        }
    }
}

/// Scaffold a new project directory and return its path.
pub fn scaffold(options: &ScaffoldOptions, catalog: &Catalog) -> Result<PathBuf, EngineError> {
    validate_project_name(&options.project)?;
    // An empty `--template` selects the catalog's first template, so a bare
    // `create-rust-app my-app` always materialises a real project.
    let template = if options.template.is_empty() {
        catalog
            .templates
            .first()
            .map(|entry| entry.slug.clone())
            .ok_or(EngineError::EmptyCatalog)?
    } else {
        options.template.clone()
    };

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

    let mut scratch_roots: Vec<PathBuf> = Vec::new();
    let result = scaffold_inner(
        options,
        catalog,
        &template,
        &target,
        &package,
        &mut scratch_roots,
    );
    for root in &scratch_roots {
        cleanup_scratch(root);
    }

    if result.is_err() && !options.keep_on_failure && target.exists() {
        let _ = fs::remove_dir_all(&target);
    }
    result
}

/// Run the scaffold steps, returning the project path plus scratch directories
/// that the caller must clean up.
fn scaffold_inner(
    options: &ScaffoldOptions,
    catalog: &Catalog,
    template: &str,
    target: &Path,
    package: &str,
    scratch_roots: &mut Vec<PathBuf>,
) -> Result<PathBuf, EngineError> {
    let (template_source, template_key) = resolve_template_source(template, catalog)?;
    let (template_dir, scratch) =
        materialise_source(&template_source, &template_key, template, options)?;
    if let Some(root) = scratch {
        scratch_roots.push(root);
    }
    fs::create_dir_all(target).map_err(|source| EngineError::ProjectWrite {
        path: target.to_string_lossy().to_string(),
        source,
    })?;
    copy_tree(&template_dir, target, template)?;

    let manifest = target.join("Cargo.toml");
    if !manifest.is_file() {
        return Err(EngineError::TemplateMaterialize {
            path: target.to_string_lossy().to_string(),
            detail: "template source has no Cargo.toml".to_string(),
        });
    }
    let old_names =
        read_manifest_names(&manifest).map_err(|source| EngineError::TemplateMaterialize {
            path: manifest.to_string_lossy().to_string(),
            detail: format!("cannot read template Cargo.toml: {source}"),
        })?;
    let snake = snake_case(package);
    rename_package(&manifest, package, &snake).map_err(|source| {
        EngineError::TemplateMaterialize {
            path: manifest.to_string_lossy().to_string(),
            detail: format!("cannot rewrite Cargo.toml: {source}"),
        }
    })?;
    merge_project_config(
        target,
        &options.project,
        template,
        &options.addons,
        &options.sets,
    )?;

    for addon in &options.addons {
        let (addon_source, addon_key) = resolve_addon_source(addon, catalog)?;
        let (addon_dir, scratch) = materialise_source(&addon_source, &addon_key, addon, options)?;
        if let Some(root) = scratch {
            scratch_roots.push(root);
        }
        let overlay = addon_dir.join("template");
        let overlay = if overlay.is_dir() { overlay } else { addon_dir };
        copy_tree(&overlay, target, addon)?;
    }
    // Point code, lockfile, and docs at the new package/library names so the
    // scaffolded project compiles without manual edits. Runs after every
    // overlay lands so addon-provided files (tests, modules) are rewritten
    // too — not just template sources.
    rewrite_package_references(target, &old_names, package, &snake).map_err(|source| {
        EngineError::TemplateMaterialize {
            path: target.to_string_lossy().to_string(),
            detail: format!("cannot rewrite package references: {source}"),
        }
    })?;
    // Best-effort `cargo fmt` so composed output stays rustfmt-clean (e.g.
    // `.append` module registrations land sorted). Never fails the scaffold:
    // rustfmt may be missing or the project may not parse in isolation.
    format_project(target);
    Ok(target.to_path_buf())
}

/// Best-effort `cargo fmt --all` on the scaffolded project.
fn format_project(target: &Path) {
    let formatted = std::process::Command::new("cargo")
        .arg("fmt")
        .arg("--all")
        .current_dir(target)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false);
    if !formatted {
        eprintln!(
            "note: post-scaffold `cargo fmt` reported issues (run `cargo fmt` in {} for details)",
            target.display()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{AddonEntry, TemplateEntry};

    /// Minimal on-disk bank: one template plus one addon overlay.
    /// The returned tempdirs must stay alive for the whole test.
    struct Fixture {
        _bank: tempfile::TempDir,
        _cache: tempfile::TempDir,
        _work: tempfile::TempDir,
        catalog: Catalog,
        options: ScaffoldOptions,
        workdir: PathBuf,
    }

    fn fixture(project: &str) -> Fixture {
        let bank = tempfile::tempdir().expect("bank tempdir");
        let root = bank.path();
        let tpl = root.join("templates").join("demo");
        fs::create_dir_all(tpl.join("src")).expect("mkdir template");
        fs::write(
            tpl.join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[[bin]]\nname = \"demo\"\npath = \"src/main.rs\"\n\n[lib]\nname = \"demo_lib\"\npath = \"src/lib.rs\"\n\n[dependencies]\n",
        )
        .expect("write manifest");
        fs::write(tpl.join("src/main.rs"), "fn main() {}\n").expect("write main");
        fs::write(tpl.join("src/lib.rs"), "// demo lib\n").expect("write lib");
        fs::write(
            tpl.join("cra.config.json"),
            "{\"name\": \"demo\", \"customOptions\": [{\"key\": \"apiPrefix\"}]}",
        )
        .expect("write config");
        let ext = root.join("extensions").join("demo-ex").join("template");
        fs::create_dir_all(&ext).expect("mkdir extension");
        fs::write(ext.join("extra.txt"), "overlay\n").expect("write overlay");

        let tpl_url = format!("file://{}", tpl.to_str().expect("utf8"));
        let ext_url = format!(
            "file://{}",
            root.join("extensions")
                .join("demo-ex")
                .to_str()
                .expect("utf8")
        );
        let catalog = Catalog {
            templates: vec![TemplateEntry {
                slug: "demo".to_string(),
                description: "Demo template".to_string(),
                tags: vec![],
                url: tpl_url,
            }],
            addons: vec![AddonEntry {
                slug: "demo-ex".to_string(),
                description: "Demo overlay".to_string(),
                url: ext_url,
            }],
        };
        let work = tempfile::tempdir().expect("work tempdir");
        let cache = tempfile::tempdir().expect("cache tempdir");
        let options = ScaffoldOptions {
            project: work.path().join(project).to_string_lossy().to_string(),
            template: "demo".to_string(),
            addons: vec![],
            sets: vec![("key".to_string(), "value".to_string())],
            force: false,
            offline: false,
            keep_on_failure: false,
            cache_dir: Some(cache.path().to_path_buf()),
            no_cache: false,
            pin: None,
        };
        let workdir = work.path().to_path_buf();
        Fixture {
            _bank: bank,
            _cache: cache,
            _work: work,
            catalog,
            options,
            workdir,
        }
    }

    #[test]
    fn rewrites_crate_references_on_rename() {
        let fixture = fixture("my-api");
        let bank_tpl = fixture
            .catalog
            .templates
            .iter()
            .find(|entry| entry.slug == "demo")
            .expect("demo template");
        let tpl = bank_tpl.url.strip_prefix("file://").expect("file url");
        // Code referencing the template's original library name, plus a
        // lockfile entry pinning the original package name.
        fs::write(
            Path::new(tpl).join("src/main.rs"),
            "use demo_lib::greet;\n\nfn main() {\n    greet();\n}\n",
        )
        .expect("write main");
        fs::write(Path::new(tpl).join("src/lib.rs"), "pub fn greet() {}\n").expect("write lib");
        fs::write(
            Path::new(tpl).join("Cargo.lock"),
            "[[package]]\nname = \"demo\"\nversion = \"0.1.0\"\ndependencies = [\n \"demo_lib\",\n]\n",
        )
        .expect("write lock");
        let target = scaffold(&fixture.options, &fixture.catalog).expect("scaffold");
        let main = fs::read_to_string(target.join("src/main.rs")).expect("read main");
        assert!(
            main.contains("use my_api::greet;"),
            "lib reference rewritten, got: {main}"
        );
        let lock = fs::read_to_string(target.join("Cargo.lock")).expect("read lock");
        assert!(
            lock.contains("name = \"my-api\""),
            "lockfile tracks the new name, got: {lock}"
        );
        assert!(!lock.contains("\"demo\""), "no stale references remain");
        let manifest = fs::read_to_string(target.join("Cargo.toml")).expect("read manifest");
        assert!(
            manifest.contains("name = \"my_api\""),
            "lib renamed, got: {manifest}"
        );
    }

    #[test]
    fn appends_fragments_to_existing_and_missing_targets() {
        let dir = tempfile::tempdir().expect("tempdir");
        let fragment = dir.path().join("router.rs.append");
        fs::write(&fragment, "pub mod auth;\n").expect("write fragment");
        let target = dir.path().join("router.rs");
        append_fragment(&fragment, &target, "test").expect("create from fragment");
        assert_eq!(
            fs::read_to_string(&target).expect("read"),
            "pub mod auth;\n"
        );
        fs::write(&target, "pub mod health;").expect("rewrite without newline");
        append_fragment(&fragment, &target, "test").expect("append");
        assert_eq!(
            fs::read_to_string(&target).expect("read"),
            "pub mod health;\npub mod auth;\n"
        );
    }

    #[test]
    fn merges_overlay_manifest_dependencies() {
        let dir = tempfile::tempdir().expect("tempdir");
        let target = dir.path().join("Cargo.toml");
        let overlay = dir.path().join("overlay.toml");
        fs::write(
            &target,
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n\n[dependencies]\nserde = \"1\"\n",
        )
        .expect("write target");
        fs::write(
            &overlay,
            "[package]\nname = \"ignored\"\n\n[dependencies]\nserde = \"1\"\njsonwebtoken = \"9\"\n",
        )
        .expect("write overlay");
        merge_manifest(&target, &overlay, "test").expect("merge");
        let merged = fs::read_to_string(&target).expect("read");
        assert!(merged.contains("name = \"demo\""), "package untouched");
        assert!(merged.contains("jsonwebtoken"), "new dep added");
        assert!(!merged.contains("ignored"), "overlay package ignored");
    }

    #[test]
    fn unions_features_on_matching_versions() {
        let dir = tempfile::tempdir().expect("tempdir");
        let target = dir.path().join("Cargo.toml");
        let overlay = dir.path().join("overlay.toml");
        fs::write(
            &target,
            "[dependencies]\ntower-http = { version = \"0.5\", features = [\"trace\"] }\n",
        )
        .expect("write target");
        fs::write(
            &overlay,
            "[dependencies]\ntower-http = { version = \"0.5\", features = [\"cors\", \"trace\"] }\n",
        )
        .expect("write overlay");
        merge_manifest(&target, &overlay, "test").expect("union");
        let merged = fs::read_to_string(&target).expect("read");
        assert!(merged.contains("\"trace\""), "keeps target features");
        assert!(merged.contains("\"cors\""), "adds overlay features");
        assert_eq!(merged.matches("tower-http").count(), 1, "single entry");
    }

    #[test]
    fn merges_bench_targets_by_name() {
        let dir = tempfile::tempdir().expect("tempdir");
        let target = dir.path().join("Cargo.toml");
        let overlay = dir.path().join("overlay.toml");
        fs::write(&target, "[package]\nname = \"demo\"\n").expect("write target");
        fs::write(&overlay, "[[bench]]\nname = \"slug\"\nharness = false\n")
            .expect("write overlay");
        merge_manifest(&target, &overlay, "test").expect("union");
        let merged = fs::read_to_string(&target).expect("read");
        assert!(merged.contains("[[bench]]"), "bench section added");
        assert!(merged.contains("harness = false"), "bench flags kept");
        // Identical re-application is a no-op (single entry).
        merge_manifest(&target, &overlay, "test").expect("idempotent");
        let merged = fs::read_to_string(&target).expect("read");
        assert_eq!(merged.matches("[[bench]]").count(), 1, "no duplicate");
    }

    #[test]
    fn rejects_conflicting_bench_targets() {
        let dir = tempfile::tempdir().expect("tempdir");
        let target = dir.path().join("Cargo.toml");
        let overlay = dir.path().join("overlay.toml");
        fs::write(
            &target,
            "[package]\nname = \"demo\"\n\n[[bench]]\nname = \"slug\"\nharness = false\n",
        )
        .expect("write target");
        fs::write(
            &overlay,
            "[[bench]]\nname = \"slug\"\npath = \"other.rs\"\n",
        )
        .expect("write overlay");
        let err = merge_manifest(&target, &overlay, "test").expect_err("conflict");
        assert!(
            matches!(err, EngineError::TemplateMaterialize { .. }),
            "conflict surfaces, got: {err:?}"
        );
    }

    #[test]
    fn rejects_conflicting_overlay_dependencies() {
        let dir = tempfile::tempdir().expect("tempdir");
        let target = dir.path().join("Cargo.toml");
        let overlay = dir.path().join("overlay.toml");
        fs::write(&target, "[dependencies]\nserde = \"1\"\n").expect("write target");
        fs::write(&overlay, "[dependencies]\nserde = \"2\"\n").expect("write overlay");
        let err = merge_manifest(&target, &overlay, "test").expect_err("conflict");
        assert!(
            matches!(err, EngineError::TemplateMaterialize { .. }),
            "conflict surfaces, got: {err:?}"
        );
    }

    #[test]
    fn applies_addon_fragments_and_manifest_merge() {
        let mut fixture = fixture("fragment-app");
        // Addon overlay: a `.append` fragment plus a dependency-only manifest.
        let addon_root = fixture
            .catalog
            .addons
            .iter()
            .find(|entry| entry.slug == "demo-ex")
            .expect("demo addon")
            .url
            .strip_prefix("file://")
            .expect("file url")
            .to_string();
        let overlay_root = Path::new(&addon_root).join("template");
        fs::write(overlay_root.join("extra.txt.append"), "more\n").expect("write fragment");
        fs::write(
            overlay_root.join("Cargo.toml"),
            "[dependencies]\nserde = \"1\"\n",
        )
        .expect("write overlay manifest");
        fixture.options.addons = vec!["demo-ex".to_string()];
        let target = scaffold(&fixture.options, &fixture.catalog).expect("scaffold");
        let extra = fs::read_to_string(target.join("extra.txt")).expect("read overlay");
        assert_eq!(extra, "overlay\nmore\n", "fragment appended, got: {extra}");
        let manifest = fs::read_to_string(target.join("Cargo.toml")).expect("read manifest");
        assert!(manifest.contains("serde"), "overlay dep merged");
    }

    #[test]
    fn rewrites_addon_provided_references() {
        let mut fixture = fixture("addon-rename-app");
        let addon_root = fixture
            .catalog
            .addons
            .iter()
            .find(|entry| entry.slug == "demo-ex")
            .expect("demo addon")
            .url
            .strip_prefix("file://")
            .expect("file url")
            .to_string();
        let overlay_root = Path::new(&addon_root).join("template");
        fs::create_dir_all(overlay_root.join("tests")).expect("mkdir tests");
        // Addon test referencing the template's original library name: the
        // post-overlay rewrite must repoint it at the new project.
        fs::write(
            overlay_root.join("tests/test_ext.rs"),
            "use demo_lib::greet;\n\n#[test]\nfn calls_lib() {\n    greet();\n}\n",
        )
        .expect("write addon test");
        fixture.options.addons = vec!["demo-ex".to_string()];
        let target = scaffold(&fixture.options, &fixture.catalog).expect("scaffold");
        let test = fs::read_to_string(target.join("tests/test_ext.rs")).expect("read test");
        assert!(
            test.contains("use addon_rename_app::greet;"),
            "addon reference rewritten, got: {test}"
        );
    }

    #[test]
    fn formats_composed_output() {
        if !std::process::Command::new("cargo")
            .arg("fmt")
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
        {
            eprintln!("skipping formats_composed_output: rustfmt unavailable");
            return;
        }
        let mut fixture = fixture("fmt-app");
        let template_root = fixture
            .catalog
            .templates
            .iter()
            .find(|entry| entry.slug == "demo")
            .expect("demo template")
            .url
            .strip_prefix("file://")
            .expect("file url")
            .to_string();
        let addon_root = fixture
            .catalog
            .addons
            .iter()
            .find(|entry| entry.slug == "demo-ex")
            .expect("demo addon")
            .url
            .strip_prefix("file://")
            .expect("file url")
            .to_string();
        // Unsorted on purpose: the fragment appends `apple` after `zebra`,
        // and the post-scaffold format must sort them. Module files exist
        // like in a real bank, otherwise rustfmt cannot resolve them.
        fs::write(
            Path::new(&template_root).join("src/lib.rs"),
            "pub mod zebra;\n",
        )
        .expect("write lib");
        fs::write(Path::new(&template_root).join("src/zebra.rs"), "").expect("write zebra");
        fs::create_dir_all(Path::new(&addon_root).join("template/src")).expect("mkdir overlay src");
        fs::write(
            Path::new(&addon_root).join("template/src/lib.rs.append"),
            "pub mod apple;\n",
        )
        .expect("write fragment");
        fs::write(Path::new(&addon_root).join("template/src/apple.rs"), "").expect("write apple");
        fixture.options.addons = vec!["demo-ex".to_string()];
        let target = scaffold(&fixture.options, &fixture.catalog).expect("scaffold");
        let lib = fs::read_to_string(target.join("src/lib.rs")).expect("read lib");
        assert_eq!(
            lib, "pub mod apple;\npub mod zebra;\n",
            "composed lib sorted, got: {lib}"
        );
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
        let fixture = fixture("my-api");
        let target = scaffold(&fixture.options, &fixture.catalog).expect("scaffold");
        assert!(target.join("Cargo.toml").is_file());
        assert!(target.join("src/main.rs").is_file());
        assert!(target.join("src/lib.rs").is_file());
        assert!(target.join("cra.config.json").is_file());
        let manifest = fs::read_to_string(target.join("Cargo.toml")).expect("read manifest");
        assert!(manifest.contains("name = \"my-api\""));
        assert!(manifest.contains("name = \"my_api\""));
        assert!(!manifest.contains("\"demo"));
        let config = fs::read_to_string(target.join("cra.config.json")).expect("read config");
        assert!(config.contains("\"project\""));
        assert!(config.contains("my-api"));
        // Template-declared keys survive the engine merge.
        assert!(config.contains("customOptions"));
    }

    #[test]
    fn applies_addon_overlay() {
        let mut fixture = fixture("with-addon");
        fixture.options.addons = vec!["demo-ex".to_string()];
        let target = scaffold(&fixture.options, &fixture.catalog).expect("scaffold");
        assert_eq!(
            fs::read_to_string(target.join("extra.txt")).expect("read overlay"),
            "overlay\n"
        );
    }

    #[test]
    fn defaults_to_first_catalog_template() {
        let mut fixture = fixture("defaulted");
        fixture.options.template = String::new();
        let target = scaffold(&fixture.options, &fixture.catalog).expect("scaffold");
        assert!(target.join("src/lib.rs").is_file());
    }

    #[test]
    fn rejects_unknown_template_and_addon() {
        let fixture = fixture("proj");
        let mut options = fixture.options.clone();
        options.template = "nope".to_string();
        let err = scaffold(&options, &fixture.catalog).expect_err("must fail");
        assert!(matches!(err, EngineError::UnknownTemplate { .. }));
        options.template = "demo".to_string();
        options.addons = vec!["nope".to_string()];
        let err = scaffold(&options, &fixture.catalog).expect_err("must fail");
        assert!(matches!(err, EngineError::UnknownAddon { .. }));
    }

    #[test]
    fn rejects_entries_without_source_url() {
        let fixture = fixture("proj");
        let mut catalog = fixture.catalog.clone();
        catalog.templates[0].url.clear();
        let err = scaffold(&fixture.options, &catalog).expect_err("must fail");
        assert!(matches!(err, EngineError::TemplateHasNoSource { .. }));
    }

    #[test]
    fn offline_supports_local_sources() {
        let mut fixture = fixture("offline-local");
        fixture.options.offline = true;
        let target = scaffold(&fixture.options, &fixture.catalog).expect("offline file:// works");
        assert!(target.join("src/lib.rs").is_file());
    }

    #[test]
    fn offline_misses_cold_remote_cache() {
        let fixture = fixture("offline-cold");
        let mut catalog = fixture.catalog.clone();
        catalog.templates[0].url = "https://example.invalid/bank?subdir=templates/demo".to_string();
        let mut options = fixture.options.clone();
        options.offline = true;
        let err = scaffold(&options, &catalog).expect_err("must fail");
        assert!(matches!(err, EngineError::OfflineCacheMiss { .. }));
    }

    #[test]
    fn offline_uses_warm_cache() {
        let fixture = fixture("offline-warm");
        // Seed the cache exactly where the engine looks for this slug.
        let cached = fixture
            .options
            .cache_dir
            .clone()
            .expect("cache dir")
            .join("banks")
            .join("template-demo");
        fs::create_dir_all(cached.join("templates/demo/src")).expect("mkdir cache");
        let cached = cached.join("templates/demo");
        fs::write(
            cached.join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
        )
        .expect("write cached manifest");
        fs::write(cached.join("src/main.rs"), "fn main() {}\n").expect("write cached main");
        let mut catalog = fixture.catalog.clone();
        catalog.templates[0].url = "https://example.invalid/bank?subdir=templates/demo".to_string();
        let mut options = fixture.options.clone();
        options.offline = true;
        let target = scaffold(&options, &catalog).expect("offline cache hit");
        let manifest = fs::read_to_string(target.join("Cargo.toml")).expect("read manifest");
        assert!(manifest.contains("name = \"offline-warm\""));
    }

    #[test]
    fn splits_subdir_queries() {
        assert_eq!(
            split_subdir("https://host/org/repo?subdir=a/b"),
            ("https://host/org/repo".to_string(), Some("a/b".to_string()))
        );
        assert_eq!(
            split_subdir("https://host/org/repo?x=1&subdir=a/"),
            (
                "https://host/org/repo?x=1".to_string(),
                Some("a".to_string())
            )
        );
        assert_eq!(
            split_subdir("https://host/org/repo"),
            ("https://host/org/repo".to_string(), None)
        );
    }

    #[test]
    fn refuses_non_empty_target_without_force() {
        let fixture = fixture("taken");
        let project_dir = fixture.workdir.join("taken");
        fs::create_dir_all(&project_dir).expect("mkdir");
        fs::write(project_dir.join("existing.txt"), "x").expect("write");
        let err = scaffold(&fixture.options, &fixture.catalog).expect_err("must fail");
        assert!(matches!(err, EngineError::TargetNotEmpty { .. }));
        let mut options = fixture.options.clone();
        options.force = true;
        scaffold(&options, &fixture.catalog).expect("force succeeds");
    }
}
