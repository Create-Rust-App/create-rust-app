<div align="center">

# Create Awesome Rust App

**Rust-native scaffolding CLI — compose templates and extensions into production-ready Rust projects.**

One command. Any Rust stack.

[![crates.io](https://img.shields.io/crates/v/create-awesome-rust-app?style=flat-square)](https://crates.io/crates/create-awesome-rust-app)
[![Tests](https://github.com/Create-Rust-App/create-rust-app/actions/workflows/test.yml/badge.svg)](https://github.com/Create-Rust-App/create-rust-app/actions/workflows/test.yml)
[![Lint](https://github.com/Create-Rust-App/create-rust-app/actions/workflows/lint.yml/badge.svg)](https://github.com/Create-Rust-App/create-rust-app/actions/workflows/lint.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](https://github.com/Create-Rust-App/create-rust-app/blob/main/LICENSE)

[Repository](https://github.com/Create-Rust-App/create-rust-app) · [Templates](https://github.com/Create-Rust-App/cra-templates) · [Releases](https://github.com/Create-Rust-App/create-rust-app/releases)

</div>

---

> **Crate name note:** the package is published as `create-awesome-rust-app`
> because `create-rust-app` is taken on crates.io by an unrelated crate. The
> installed binary is still `create-rust-app` (with a `create-awesome-rust-app`
> alias).

## Install

```bash
cargo install create-awesome-rust-app
```

Other channels:

| Channel | How |
|---------|-----|
| Install script | `curl -fsSL https://create-awesome-rust-app.vercel.app/install.sh \| sh` |
| Homebrew | `brew tap Create-Rust-App/tap && brew install create-awesome-rust-app` |
| AUR (source) | `yay -S create-awesome-rust-app` |
| AUR (prebuilt) | `yay -S create-awesome-rust-app-bin` |
| GitHub Release | Prebuilt binaries + `SHA256SUMS` on the [releases page](https://github.com/Create-Rust-App/create-rust-app/releases) |

## Quick start

```bash
create-rust-app my-app --template axum-starter --addons all-github-setup
# or
create-awesome-rust-app my-app --template axum-starter --addons all-github-setup
```

Headless / CI:

```bash
create-rust-app my-api \
  --template axum-starter \
  --addons all-github-setup \
  --no-interactive --no-install
```

List what is available in the catalog:

```bash
create-rust-app --list-templates
create-rust-app --list-addons
```

## How it works

- Templates and addons resolve from catalog slugs (for example `axum-starter`),
  `file://` directories, or git URLs with an optional `?subdir=` selector.
- Fetched content is cached under the scaffold cache; `--offline` reuses the
  cache and `--pin` pins a git ref.
- `Cargo.toml` is renamed to the new project, and references to the template's
  original package/library names are rewritten across code, `Cargo.lock`, and
  docs — the scaffolded project passes `cargo check` without manual edits.
- A `cra.config.json` project record is merged into the new project
  (project/template/addons plus any `--set key=value` overrides).
- Unless `--no-install` is passed, a post-scaffold `cargo check` runs as a
  best-effort verification step.

Useful flags: `--force`, `--offline`, `--no-cache`, `--cache-dir`,
`--catalog-url` / `--catalog-path`, `--category`, `--json`,
`cache status|clean`, shell completions via `--add-completion`.

Template and extension **content** lives in
[`cra-templates`](https://github.com/Create-Rust-App/cra-templates)
(`--catalog-path` and `file://` URLs support local forks).

## Development

Requires Rust (pinned in `rust-toolchain.toml`):

```bash
make build      # debug binary -> ./target/debug/create-rust-app
make test       # cargo test --workspace --locked
make clippy     # clippy with -D warnings
make fmt-check  # rustfmt check
```

## Contributing

See [CONTRIBUTING.md](https://github.com/Create-Rust-App/create-rust-app/blob/main/CONTRIBUTING.md).

## License

MIT — see [LICENSE](https://github.com/Create-Rust-App/create-rust-app/blob/main/LICENSE).
