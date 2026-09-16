//! Scaffolding engine for Create Rust App.
//!
//! The engine mirrors the sibling `create-vlang-app` / `create-python-app`
//! cores: it loads a template catalog (`templates.json`), resolves template
//! and addon slugs, and materialises a new Rust project on disk. Template
//! and extension **content** lives in
//! [`cra-templates`](https://github.com/Create-Rust-App/cra-templates); the
//! CLI consumes
//!
//! ```text
//! https://raw.githubusercontent.com/Create-Rust-App/cra-templates/main/templates.json
//! ```

mod cache;
mod catalog;
mod info;
mod scaffold;

pub use cache::{cache_dir, clean_cache};
pub use catalog::{
    format_catalog_list, list_addon_names, list_template_names, list_template_names_in_category,
    load_catalog, resolve_fixture_catalog_path, AddonEntry, Catalog, TemplateEntry,
    DEFAULT_CATALOG_URL,
};
pub use info::{brand_banner, env_info};
pub use scaffold::{parse_set_override, scaffold, validate_project_name, ScaffoldOptions};
