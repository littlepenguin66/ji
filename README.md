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

## Quick Start

```bash
# Install
cargo install --path .

# Initialize
ji init                        # interactive age keypair generation
ji init -a                     # skip prompts, auto-generate
ji init --key ~/.ssh/id_ed25519.pub  # use existing SSH key

# Add files
ji add .zshrc .gitconfig .config/nvim/
ji add **/* --exclude "*.zwc"

# Pack
ji pack                        # → ~/.local/share/ji/<hostname>.ji

# Verify
ji check mbp.ji                # fast: list files + verify HMAC
ji check mbp.ji --deep         # full: decrypt and verify all checksums

# Restore
ji unpack mbp.ji               # skip existing files
ji unpack mbp.ji --backup      # backup existing files as .bak first
ji unpack mbp.ji --dry-run     # preview without writing
```

## Usage

### Local

| Command | Description |
|---|---|
| `ji init [--key <K>]... [-a] [--force]` | Initialize config and generate age keypair |
| `ji add <PATH>... [--include <P>] [--exclude <P>]` | Add files to tracking manifest |
| `ji rm <PATH>... [--all]` | Remove files from manifest |
| `ji list [--json]` | List tracked files and checksums |
| `ji status [-s]` | Show file change status (M=modified, -=deleted, A=missing) |
| `ji diff [<PATH>]` | Show unified diff (requires prior pack for cache) |
| `ji pack [-o <PATH>] [--strict] [--verbose]` | Pack into encrypted .ji file |
| `ji unpack <INPUT> [--force\|--backup\|--interactive]` | Decrypt and restore |
| `ji check <INPUT> [--deep]` | Verify .ji integrity |

### Remote

| Command | Description |
|---|---|
| `ji remote add <NAME> --type webdav --url <URL> [--user <U>]` | Add remote endpoint |
| `ji remote remove <NAME>` | Remove endpoint |
| `ji remote list [--json]` | List all endpoints |
| `ji remote test <NAME>` | Test connectivity |
| `ji remote files <NAME>` | List .ji files on remote |
| `ji remote delete <NAME> <FILE>` | Delete .ji from remote |
| `ji push <NAME> <INPUT>` | Push .ji to remote |
| `ji pull <NAME>` | Pull .ji from remote |
| `ji sync <NAME>` | Bidirectional sync (pack+push if local changes, else pull) |

```bash
# Add a WebDAV endpoint
ji remote add nas --type webdav --url https://nas.local/ji/ --user jrz

# Push
ji pack && ji push nas mbp.ji

# Pull on a new machine
ji pull nas && ji unpack mbp.ji

# Or just sync
ji sync nas
```

### Recipients

| Command | Description |
|---|---|
| `ji recipient list <INPUT>` | List all recipients of a .ji file |
| `ji recipient add --key <PUBKEY> <INPUT>` | Add a recipient (requires decryption ability) |
| `ji recipient remove --key <PUBKEY> <INPUT>` | Remove a recipient |

```bash
# Desktop packs, laptop wants to decrypt
ji recipient add --key ~/.ssh/laptop.pub mbp.ji
# Now both machines can decrypt mbp.ji with their own keys
```

### Other

| Command | Description |
|---|---|
| `ji completion <SHELL>` | Generate shell completion script |

```bash
ji completion fish > ~/.config/fish/completions/ji.fish
```

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

```
┌──────────────────────────────┐
│ magic: 0xE6 0xAC 0x88        │  ← "笈" in UTF-8 (3 bytes)
│ version: u8 (1)               │
│ cipher: u8 (0=age, 1=pgp)    │
│ index_len: u32                │
├──────────────────────────────┤
│ plaintext index (HMAC-SHA256) │  ← inspectable without decryption
├──────────────────────────────┤
│ encrypted payload             │
│   encrypt( zstd( tar(...) )) │
└──────────────────────────────┘
```

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

**How do I switch machines?** `ji pack` on the old machine → transfer the `.ji` file → `ji init --key <new key>` → `ji unpack`. If you're the same person, add the new machine's key as a recipient first: `ji recipient add --key ~/.ssh/new.pub old.ji`.

## License

MIT

---

If ji helps you, give it a star. Bugs or ideas? Open an issue.

[中文文档](docs/README_CN.md)
