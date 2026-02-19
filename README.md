# cargo-merge-assist

Semantic merge assistant for Rust dependency files:

- **`Cargo.toml`**: 3-way semantic merge (base / ours / theirs)
- **`Cargo.lock`**: deterministic regeneration via Cargo
- Optional post-merge verification with `cargo check -q`

This project is designed for teams that frequently hit merge conflicts in dependency files.

---

## Why this exists

`Cargo.lock` and `Cargo.toml` conflicts are common in active repos with many PR rebases. Existing workflows are often manual and error-prone.

`cargo-merge-assist` provides a practical **resolve-then-verify** workflow:

1. Merge manifest changes semantically (fail fast on real semantic conflicts)
2. Regenerate lockfile from source of truth (`Cargo.toml`)
3. Verify build graph quickly (`cargo check -q`)

---

## Install

```bash
cargo install --path .
```

Or build locally:

```bash
cargo build --release
```

---

## Commands

### 1) Merge `Cargo.toml`

```bash
cargo-merge-assist merge-manifest \
  --base /tmp/base.Cargo.toml \
  --ours /tmp/ours.Cargo.toml \
  --theirs /tmp/theirs.Cargo.toml \
  --out /tmp/ours.Cargo.toml
```

Semantics:

- If one side changed and the other stayed at base → changed side wins
- If both sides changed the same value → accepted
- If both sides changed differently → conflict with key path (e.g. `dependencies.serde`)

### 2) Regenerate `Cargo.lock`

```bash
cargo-merge-assist resolve-lock --repo . --verify
```

- Runs `cargo generate-lockfile`
- Optionally runs `cargo check -q` when `--verify` is used

Offline mode:

```bash
cargo-merge-assist resolve-lock --repo . --verify --offline
```

### 3) End-to-end flow

```bash
cargo-merge-assist merge-all \
  --base /tmp/base.Cargo.toml \
  --ours /tmp/ours.Cargo.toml \
  --theirs /tmp/theirs.Cargo.toml \
  --out /tmp/ours.Cargo.toml \
  --repo .
```

This performs manifest merge + lockfile regeneration + verification.

### 4) Install local git merge drivers

```bash
cargo-merge-assist install-git-driver --repo .
```

This updates:

- `.gitattributes`
  - `Cargo.toml merge=cargo-merge-assist-manifest`
  - `Cargo.lock merge=cargo-merge-assist-lock`
- `.git/config`
  - merge driver definitions for manifest and lockfile

> Merge driver is local (`.git/config`) by design.

---

## CI Quality Gates

The repository includes a CI workflow that runs:

- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test`

---

## Current scope / limitations

- `Cargo.toml` merge is semantic but intentionally strict: divergent edits to the same scalar key will fail fast.
- Comments/formatting in merged manifest are not preserved exactly (semantic content is preserved).
- Lockfile strategy relies on Cargo regeneration (source of truth is the manifest).

---

## License

MIT
