# 笈 (ji) Agent Guide

This file is the **map** for AI agents working on this project — keep it short (~80 lines). Design details live in `.claude/design/MVP.md`.

## 1. What This Is

ji is a dotfiles management tool. It encrypts config files into a single `.ji` file for safe cross-device migration. Think lightweight chezmoi: source files → encrypted archive → single-file transfer. No Git, no symlinks.

- [README](README.md) — user-facing docs

## 2. Architecture Contract

```text
commands/  (leaf — orchestration only, no direct crate calls)
├── store/     → filesystem I/O (config, manifest, ignore, XDG paths)
├── crypto/    → Cipher trait + age/pgp backends
├── archive/   → .ji file read/write (wraps tar+zstd+crypto+hmac)
└── remote/    → Remote trait + webdav/ssh backends
```

Non-negotiable:
- `commands/` never imports `tar`, `zstd`, `rage`, `sha2`, `reqwest`, or `russh` directly
- `archive/` is the only entry point for `.ji` file format
- `store/path.rs` has zero internal dependencies
- `remote/` handles transport only — never touches file contents

## 3. Commands

```bash
# Format
cargo fmt --all -- --check

# Strict lint
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Test
cargo test                        # Default features
cargo test --all-features         # All features (needs nettle for pgp)

# Build individual features
cargo build                       # Default: age + WebDAV
cargo build --features pgp        # + PGP (needs nettle)
cargo build --features ssh        # + SSH
```

## 4. Verification Gates

Before every commit, run:

```bash
cargo fmt --all -- --check && \
  cargo clippy --workspace --all-targets --all-features -- -D warnings && \
  cargo test --all-features
```

If `--all-features` fails due to missing system deps (nettle), at minimum run:

```bash
cargo fmt --all -- --check && \
  cargo clippy --workspace --all-targets -- -D warnings && \
  cargo test
```

### Pre-Commit Hook

```bash
cp scripts/pre-commit .git/hooks/pre-commit && chmod +x .git/hooks/pre-commit
```

## 5. Non-Negotiable Rules

- `commands/` orchestration only — no direct crypto/archive crate calls
- Archive writes are always atomic: tmp → fsync → rename
- Passwords are never stored on disk
- All new modules must have tests
- Never commit credentials, tokens, or private keys
- Strict lint and all-feature tests pass at every commit
- Keep `CLAUDE.md` and `AGENTS.md` mirrored in the same change

## 6. Implementation Order

When building new features, follow the layered order:

1. Traits and abstractions first (cipher, remote)
2. Core modules next (archive, store)
3. Commands last (orchestration)

New cipher or remote backends go behind feature flags.

## 7. Where to Look Next

| Need | Go to |
|---|---|
| User-facing docs | `README.md` |
| CLI reference | `docs/cli.md` |
| Architecture | `docs/architecture.md` |
| Troubleshooting | `docs/troubleshooting.md` |
| Release notes | `docs/release/` |
| Error type catalog | `src/error.rs` |
| Cipher trait | `src/crypto/mod.rs` |
| Remote trait | `src/remote/mod.rs` |
| .ji binary format | `src/archive/format.rs` |
| CLI definition | `src/main.rs` |
