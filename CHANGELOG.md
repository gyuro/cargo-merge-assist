# Changelog

All notable changes to this project will be documented in this file.

## [0.1.0] - 2026-02-19

### Added
- Initial `cargo-merge-assist` CLI with commands:
  - `merge-manifest` (semantic 3-way merge for `Cargo.toml`)
  - `resolve-lock` (regenerate `Cargo.lock`)
  - `merge-all` (merge + lock regeneration + optional verify)
  - `install-git-driver` (local git merge driver setup)
- Semantic merge engine for TOML manifests with conflict path reporting.
- Unit tests for merge behavior and conflict detection.
- Project documentation (`README.md`, `CONTRIBUTING.md`).
- Rust CI workflow (fmt, clippy, test).
- MIT license.
- Local quality automation via `Makefile` (`make check`).

### Notes
- `Cargo.toml` merge intentionally fails fast on true conflicting edits to the same key.
- Formatting/comments are normalized by TOML serialization.
