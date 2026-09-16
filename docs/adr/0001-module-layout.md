# ADR 0001 — Cargo workspace module layout

## Status

Accepted.

## Context

Sibling CLIs keep the scaffolding engine separate from the CLI binary
(`modules/create_vlang_app_core` + `modules/create_vlang_app` in V;
`packages/*-core` + binary packages in Python/Node). The Rust port follows
the same split using a Cargo workspace.

## Decision

- `crates/create_rust_app_core` — library: catalog loading, addon/template
  resolution, project materialisation, cache helpers, environment info.
- `crates/create_rust_app` — binary: `clap` argument parsing, `cache`
  subcommand, shell completion generation, orchestration.
- `fixtures/catalog/templates.json` — offline catalog used by tests and
  `--fixture` runs.
- `completions/` — checked-in shell completions generated from the binary.
- Toolchain pinned in `rust-toolchain.toml` (analog of sibling `.v-version`).
