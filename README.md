<div align="center">

# Create Rust App

**Rust-native scaffolding CLI — compose templates and extensions into production-ready Rust projects.**

One command. Any Rust stack.

[![Tests](https://github.com/Create-Rust-App/create-rust-app/actions/workflows/test.yml/badge.svg)](https://github.com/Create-Rust-App/create-rust-app/actions/workflows/test.yml)
[![Lint](https://github.com/Create-Rust-App/create-rust-app/actions/workflows/lint.yml/badge.svg)](https://github.com/Create-Rust-App/create-rust-app/actions/workflows/lint.yml)
[![Release](https://img.shields.io/github/v/release/Create-Rust-App/create-rust-app?filter=create-rust-app%40*&style=flat-square&label=Release)](https://github.com/Create-Rust-App/create-rust-app/releases)
[![Crates.io](https://img.shields.io/crates/v/create-awesome-rust-app.svg?style=flat-square)](https://crates.io/crates/create-awesome-rust-app)
[![Crates.io Downloads](https://img.shields.io/crates/d/create-awesome-rust-app.svg?style=flat-square)](https://crates.io/crates/create-awesome-rust-app)
[![AUR](https://img.shields.io/aur/version/create-awesome-rust-app?style=flat-square&label=AUR&logo=archlinux)](https://aur.archlinux.org/packages/create-awesome-rust-app)
[![Homebrew](https://img.shields.io/badge/homebrew-Create--Rust--App%2Ftap-orange?style=flat-square&logo=homebrew)](https://github.com/Create-Rust-App/homebrew-tap)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](LICENSE)
[![Discord](https://img.shields.io/discord/1527933660764831825?style=flat-square&label=Discord&logo=discord&logoColor=white)](https://discord.gg/bR5VyATgka)

[Catalog](https://github.com/Create-Rust-App/cra-templates) · [Contributing](CONTRIBUTING.md) · [Releases](https://github.com/Create-Rust-App/create-rust-app/releases)

</div>

---

## Quick start

### Install (recommended)

```bash
curl -fsSL https://create-awesome-rust-app.vercel.app/install.sh | sh
```

Installs `create-rust-app` and a `create-awesome-rust-app` alias into `~/.local/bin`
(override with `CRA_INSTALL_DIR`). Pin a version with `CRA_VERSION=0.4.0`.

Then scaffold:

```bash
create-rust-app my-app --template axum-starter --addons all-github-setup
# or
create-awesome-rust-app my-app --template axum-starter --addons all-github-setup
```

Fallback (raw script from this repo):

```bash
curl -fsSL https://raw.githubusercontent.com/Create-Rust-App/create-rust-app/main/scripts/install.sh | sh
```

### Manual / pin a Release asset

Download a specific platform binary from the Releases page:

```bash
curl -fsSL -o create-rust-app \
  "https://github.com/Create-Rust-App/create-rust-app/releases/download/create-rust-app%400.4.0/create-rust-app-linux-x86_64"
chmod +x create-rust-app
mv create-rust-app ~/.local/bin/
```

### Build from source

Requires Rust (pinned in [`rust-toolchain.toml`](rust-toolchain.toml)):

```bash
git clone https://github.com/Create-Rust-App/create-rust-app.git
cd create-rust-app
make build
./target/debug/create-rust-app --help
```

Headless / CI:

```bash
create-rust-app my-api \
  --template axum-starter \
  --addons all-github-setup \
  --no-interactive --no-install
```

List catalog entries:

```bash
create-rust-app --list-templates
create-rust-app --list-addons
```

## What this repo contains

| Path | Purpose |
|------|---------|
| [`crates/create_rust_app_core`](crates/create_rust_app_core) | Scaffolding engine (catalog, fetch, merge, install) |
| [`crates/create_rust_app`](crates/create_rust_app) | CLI binary |
| [`docs/`](docs) | ADRs, distribution notes |

Template and extension **content** lives in [`cra-templates`](https://github.com/Create-Rust-App/cra-templates). The CLI consumes:

```text
https://raw.githubusercontent.com/Create-Rust-App/cra-templates/main/templates.json
```

Override with `--catalog-path` or a fork for local testing (`file://` supported).

## Ecosystem

| Repository | Role |
|------------|------|
| [create-rust-app](https://github.com/Create-Rust-App/create-rust-app) (this repo) | CLI + core engine |
| [cra-templates](https://github.com/Create-Rust-App/cra-templates) | Official templates and extensions |
| [website](https://github.com/Create-Rust-App/website) | Docs / catalog UI |
| [homebrew-tap](https://github.com/Create-Rust-App/homebrew-tap) | Homebrew formula |
| [aur-package](https://github.com/Create-Rust-App/aur-package) | AUR PKGBUILD mirror |

## Distribution

Tagged `create-rust-app@X.Y.Z` releases draft a GitHub Release with a
five-target binary matrix (Linux x86_64 required; Linux arm64, macOS arm64,
macOS x86_64, and Windows x86_64 best-effort) plus `SHA256SUMS`, and publish
both crates to [crates.io](https://crates.io/crates/create-awesome-rust-app)
via Trusted Publishing (no stored tokens).

## License

MIT — see [LICENSE](LICENSE).
