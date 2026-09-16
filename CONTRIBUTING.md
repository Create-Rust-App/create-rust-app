# Contributing

Thanks for helping with Create Rust App. English for commits, PRs, issues, and docs.

## Workflow

1. Open or pick an issue first; link it from your PR (`Closes #N`).
2. Never commit directly to `main`; work on a short-lived branch.
3. One fix per PR; keep diffs minimal.
4. Before opening the PR, run the full local gate:

```bash
make fmt-check
make clippy
make test
```

## Release process

Maintainers tag `create-rust-app@X.Y.Z`. The `Release` workflow verifies the
tag matches `crates/create_rust_app/Cargo.toml`, builds the binary matrix,
and drafts a GitHub Release with checksums. Nothing is published to crates.io
yet, even though package metadata is complete.
