# 笈 (ji)

*Pack your dotfiles into a bamboo case and go.*

> 昔者负笈游，今者云笈收。散珠归宝椟，代代可相酬。
> *Once, scholars wandered with bamboo cases on their backs. Now, the cloud carries your scrolls.*

## Why

Ancient scholars carried a 笈 — a bamboo book-case — filled with hand-copied manuscripts. That case held everything they needed to work, wherever they went.

Your `.zshrc`, `.vimrc`, `.gitconfig` are today's manuscripts. **ji** is your bamboo case — encrypt your dotfiles into a single `.ji` file and carry them safely between machines. Open the case, and you're home.

Think of it as a lightweight chezmoi: source files → encrypted archive → single-file migration. No Git, no symlinks.

## Features

- **Single-file migration** — all dotfiles packed into one self-contained `.ji` file
- **age encryption** — supports SSH keys and age keypairs, multiple recipients
- **Optional PGP** — `--features pgp` enables sequoia-pgp backend for GnuPG users
- **Remote sync** — WebDAV first class; SSH via feature flag
- **Integrity verification** — HMAC-SHA256 on the file index, SHA-256 per-file checksums
- **Atomic writes** — tmp → fsync → rename everywhere, Ctrl+C leaves nothing broken
- **Unix philosophy** — each command does one thing, `--json` for scripting
- **Shell completions** — bash / zsh / fish with dynamic argument completion

## Install

```bash
cargo install ji          # from crates.io
```

Or build from source:

```bash
git clone https://github.com/your/ji.git && cd ji && cargo install --path .
```

Pre-built binaries and Homebrew tap coming soon.

## Quick Start

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
ji sync nas                     # auto-detect direction & pack/push or pull
```

### Diagnose issues

```bash
ji doctor                       # config + keys + manifest
ji doctor --full                # add remote connectivity + archive scan
ji doctor --json | jq .checks   # scriptable output
```

## Docs

| Document | What it covers |
|---|---|
| [CLI Reference](docs/cli.md) | Every command, every option |
| [Architecture](docs/architecture.md) | .ji file format, layers, encryption flow |
| [Troubleshooting](docs/troubleshooting.md) | Doctor-guided diagnosis and fixes |

## Config

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

Passwords are never stored — WebDAV / SSH prompts each time. `.jiignore` at `~/.config/ji/.jiignore` uses gitignore syntax; defaults exclude `.ssh/`, `.DS_Store`, `node_modules/`.

## .ji File Format

A `.ji` file is `encrypt( zstd( tar(manifest + files) ))` — with a plaintext HMAC-signed index prepended so `ji list` and `ji check` work without decryption. Full spec in [Architecture](docs/architecture.md).

## Feature Flags

```bash
cargo build                   # default: age + WebDAV
cargo build --features pgp    # + PGP encryption (needs nettle: brew install nettle)
cargo build --features ssh    # + SSH remote transport
```

## FAQ

**How is this different from chezmoi?** chezmoi relies on Git as the transport layer and encrypts files individually. ji packs everything into a single encrypted `.ji` file — self-contained, no Git dependency, no leaked metadata.

**Why not just use Git?** Git leaks file names, sizes, and change frequency. ji's `.ji` file is an opaque encrypted blob. Git is for version history; ji is for secure migration.

**Where are passwords stored?** Nowhere. ji prompts for passwords on every push/pull and never touches your credentials.

**Windows support?** V1 targets macOS and Linux only.

**How do I switch machines?** See the cookbook above: pack on the old machine, transfer the `.ji` file, init on the new machine with your SSH key, then unpack. If you use multiple devices regularly, add each one's key as a recipient so any device can decrypt the same `.ji` file.

## License

MIT

---

If ji helps you, give it a star. Bugs or ideas? Open an issue.

[中文文档](docs/README_CN.md)
