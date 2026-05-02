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
```

### First time: set up and pack

```bash
# 1. Initialize — creates config + generates age keypair
ji init

# 2. Add the dotfiles you want to carry
ji add .zshrc .gitconfig .config/nvim/
ji add **/* --exclude "*.zwc"

# 3. Check what's tracked
ji list
ji status

# 4. Pack into an encrypted .ji file
ji pack
# → ~/.local/share/ji/mbp.ji
```

### Switch to a new machine

```bash
# 1. Bring your .ji file over (USB, scp, cloud...)
# 2. Initialize ji on the new machine
ji init --key ~/.ssh/id_ed25519.pub

# 3. Preview what will be restored
ji unpack mbp.ji --dry-run

# 4. Restore
ji unpack mbp.ji --backup

# 5. Check everything looks right
ji status
```

### Share access between your devices

```bash
# Desktop packs, laptop wants to decrypt too
ji recipient add --key ~/.ssh/laptop.pub mbp.ji
# Now both machines can decrypt mbp.ji with their own key
```

### Sync with a remote (WebDAV NAS)

```bash
# One-time setup
ji remote add nas --type webdav --url https://nas.local/ji/ --user jrz
ji remote test nas

# Push to remote
ji pack && ji push nas mbp.ji

# Pull on another machine
ji pull nas
ji unpack mbp.ji

# Or just sync (detects direction automatically)
ji sync nas
```

### Check the health of your setup

```bash
ji doctor           # fast: config + keys + manifest
ji doctor --full    # everything + remote + archives
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
