# AGENTS.md — create-rust-app

## Purpose

Rust-native scaffolding CLI (`create-rust-app`) and engine (`create_rust_app_core`).

## Hard rules

1. **Rust-native only** — do not introduce Node/Python as the scaffolding runtime without an ADR.
2. **Issue-first** — every significant change links a GitHub issue (`Closes #N`).
3. **One fix per PR** — ready for review (no drafts unless requested).
4. **English** for commits, PRs, issues, and docs.
5. **Never commit directly to `main`** (except empty-repo bootstrap).
6. **Never leave `main` red.**
7. Prefer minimal diffs.

## Layout

See `docs/adr/0001-module-layout.md`. Crates live under `crates/`.

## Commands

```bash
make test
make fmt-check
make clippy
make build
```
