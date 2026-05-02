# 笈 (ji)

*Pack your dotfiles into a bamboo case and go.*

[![Rust](https://img.shields.io/badge/Rust-2024-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-AGPL%20v3-blue?style=flat-square)](LICENSE)

> 昔者负笈游，今者云笈收。散珠归宝椟，代代可相酬。
> *Once, scholars wandered with bamboo cases on their backs. Now, the cloud carries your scrolls.*

---

## Overview

Ancient scholars carried a 笈 — a bamboo book-case filled with hand-copied manuscripts. That case held everything they needed to work, wherever they went.

ji is your bamboo case for the digital age. It encrypts your dotfiles into a single `.ji` file and carries them safely between machines. Think of it as a lightweight chezmoi: source files → encrypted archive → single-file migration. No Git, no symlinks.

Open the case, and you're home.

## Features

- **Single-file migration** — all dotfiles packed into one self-contained `.ji` file
- **age encryption** — supports SSH keys and age keypairs, multiple recipients, any one can decrypt
- **Remote sync** — WebDAV first class; SSH via feature flag (`--features ssh`)
- **Optional PGP** — `--features pgp` enables sequoia-pgp backend for GnuPG users
- **Integrity verification** — HMAC-SHA256 on the file index, SHA-256 per-file checksums
- **Atomic writes** — tmp → fsync → rename everywhere, Ctrl+C leaves nothing broken
- **Unix philosophy** — each command does one thing, `--json` for scripting, concise output
- **Shell completions** — bash / zsh / fish with dynamic argument completion

## Installation

### From Crates.io

```bash
cargo install ji-cli
```

### From Source

```bash
git clone https://github.com/littlepenguin66/ji.git
cd ji
cargo install --path .
```

Pre-built binaries and Homebrew tap coming soon.

## Usage

### Set up a new machine

```bash
ji init                         # generates age keypair, creates config
ji add .zshrc .gitconfig .config/nvim/
ji add **/* --exclude "*.zwc"   # bulk add with filters
ji list                         # see what's tracked (the "manifest")
ji status                       # check for uncommitted changes
ji pack                         # → ~/.local/share/ji/<hostname>.ji
```

`ji list` shows the manifest — the set of files ji tracks. `ji pack` names the output after your hostname (`uname -n`), so each machine naturally produces a different `.ji` file.

### Restore on another machine

```bash
ji init --key ~/.ssh/id_ed25519.pub
ji unpack mbp.ji --dry-run      # preview first
ji unpack mbp.ji --backup       # restore, backup existing files
ji status                       # verify
```

### Work across multiple devices

Pack once, then add each device's key so any machine can decrypt:

```bash
ji pack                         # on desktop
ji recipient add --key ~/.ssh/laptop.pub desktop.ji
ji recipient add --key ~/.ssh/server.pub desktop.ji
ji recipient list desktop.ji   # verify all keys
```

You must hold at least one existing private key to add or remove recipients. Each device uses its own SSH key — no shared secrets.

### Sync via WebDAV

```bash
ji remote add nas --type webdav --url https://nas.local/ji/ --user jrz
ji remote test nas
ji pack && ji push nas mbp.ji   # upload

ji pull nas && ji unpack mbp.ji # download on another machine
ji sync nas                     # auto-detect direction: pack+push or pull
```

### Diagnose issues

```bash
ji doctor                       # config + keys + manifest
ji doctor --full                # add remote connectivity + archive scan
ji doctor --json | jq .checks   # scriptable output
```

## How It Works

ji follows a simple pipeline:

1. **Track** — `ji add` records which dotfiles to manage in a manifest
2. **Pack** — tar the files, compress with zstd, encrypt with age, wrap in a `.ji` container
3. **Transfer** — move the `.ji` file however you like (USB, scp, WebDAV, cloud)
4. **Unpack** — decrypt, decompress, restore — atomically, with conflict protection

The `.ji` file is `encrypt( zstd( tar(manifest + files) ))` — with a plaintext HMAC-signed index prepended so `ji list` and `ji check` work without decryption.

## Commands

| Category | Commands |
|---|---|
| Local | `init`, `add`, `rm`, `list`, `status`, `diff` |
| Archive | `pack`, `unpack`, `check` |
| Remote | `remote add/remove/list/test/files/delete`, `push`, `pull`, `sync` |
| Security | `recipient list/add/remove` |
| Diagnostics | `doctor` |
| Developer | `completion` |

Full reference: [CLI Reference](docs/cli.md)

## Configuration

```toml
# ~/.config/ji/config.toml

[encryption]
type = "age"                          # age | pgp
recipients = ["ssh-ed25519 AAAAC3...", "age1..."]

[[remote]]
name = "nas"
type = "webdav"
url = "https://nas.local/ji/"
user = "jrz"
```

Passwords are never stored — WebDAV / SSH prompts each time.

## Ignoring Files

`.jiignore` at `~/.config/ji/.jiignore` uses gitignore syntax. Default excludes:

| Pattern | Reason |
|---|---|
| `.ssh/` | Private keys should never leave the machine |
| `.DS_Store` | macOS metadata |
| `node_modules/` | Dependencies, not config |

## Troubleshooting

Run `ji doctor` first — it diagnoses config, keys, manifest, and remote connectivity in one command.

| Issue | Fix |
|---|---|
| `no recipients configured` | `ji init --key ~/.ssh/id_ed25519.pub` |
| `no private key available` | Check `~/.local/share/ji/` or `~/.ssh/` for keys |
| `HMAC verification failed` | File is corrupted; re-download or re-pack |
| `push failed: 401` | Wrong WebDAV credentials |
| `connection refused` | Check remote URL with `ji remote test` |

Full guide: [Troubleshooting](docs/troubleshooting.md)

## Feature Flags

```bash
cargo build                   # default: age + WebDAV
cargo build --features pgp    # + PGP encryption (brew install nettle)
cargo build --features ssh    # + SSH remote transport
```

## Requirements

- **Rust**: 1.80+ (for building from source)
- **PGP feature**: `nettle` system library (`brew install nettle` on macOS, `apt install libnettle-dev` on Linux)

## Docs

| Document | What it covers |
|---|---|
| [CLI Reference](docs/cli.md) | Every command, every option |
| [Architecture](docs/architecture.md) | .ji file format, layers, encryption flow |
| [Troubleshooting](docs/troubleshooting.md) | Doctor-guided diagnosis and fixes |
| [Security](SECURITY.md) | Threat model and vulnerability reporting |
| [Release Notes](docs/release/) | Per-version changelogs |

## FAQ

**How is this different from chezmoi?** chezmoi relies on Git as the transport layer and encrypts files individually. ji packs everything into a single encrypted `.ji` file — self-contained, no Git dependency, no leaked metadata.

**Why not just use Git?** Git leaks file names, sizes, and change frequency. ji's `.ji` file is an opaque encrypted blob. Git is for version history; ji is for secure migration.

**Where are passwords stored?** Nowhere. ji prompts for passwords on every push/pull and never touches your credentials.

**Windows support?** V1 targets macOS and Linux only.

## License

AGPL-3.0

---

If ji helps you, give it a star. Bugs or ideas? Open an issue.

[中文文档](docs/README_CN.md)
