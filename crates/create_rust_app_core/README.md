<div align="center">

# create_rust_app_core

**Scaffolding engine behind [Create Awesome Rust App](https://crates.io/crates/create-awesome-rust-app) — catalog, fetch, merge, install.**

[![crates.io](https://img.shields.io/crates/v/create_rust_app_core?style=flat-square)](https://crates.io/crates/create_rust_app_core)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](https://github.com/Create-Rust-App/create-rust-app/blob/main/LICENSE)

[Repository](https://github.com/Create-Rust-App/create-rust-app) · [Templates](https://github.com/Create-Rust-App/cra-templates)

</div>

---

This library implements the project-scaffolding pipeline used by the
`create-rust-app` CLI:

- Resolve templates and addons from catalog slugs, `file://` directories, or
  git URLs (sparse checkout, optional `?subdir=`, optional `--pin`).
- Cache fetched content with offline reuse and configurable refresh policies.
- Materialise the template into a new project: rename `Cargo.toml` to the new
  package, rewrite stale package/library references across code, `Cargo.lock`,
  and docs, and merge the `cra.config.json` project record.

## Use

```rust
use create_rust_app_core::{load_catalog, scaffold, ScaffoldOptions};

let options = ScaffoldOptions {
    project: "my-api".to_string(),
    template: "axum-starter".to_string(),
    addons: vec!["all-github-setup".to_string()],
    sets: vec![],
    force: false,
    offline: false,
    keep_on_failure: false,
    cache_dir: None,
    no_cache: false,
    pin: None,
};
let catalog = load_catalog(None, None)?;
let target = scaffold(&options, &catalog)?;
```

Key items: `scaffold`, `ScaffoldOptions`, `Catalog` (`TemplateEntry`,
`AddonEntry`), `cache_dir` / `clean_cache`, `validate_project_name`,
`parse_set_override`, `brand_banner` / `env_info`.

## Development

```bash
make test    # cargo test --workspace --locked
make clippy  # clippy with -D warnings
```

## License

MIT — see [LICENSE](https://github.com/Create-Rust-App/create-rust-app/blob/main/LICENSE).
