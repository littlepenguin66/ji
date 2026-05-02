# CLI Reference

## ji init

Initialize ji configuration and generate an age keypair.

```
ji init [--key <PUBKEY>]... [-a] [--force]
```

| Option | Description |
|---|---|
| `--key <PUBKEY>` | Add a public key as recipient (repeatable) |
| `-a` | Auto-generate age keypair without prompts |
| `--force` | Overwrite existing config.toml |

Creates `~/.config/ji/config.toml` and `~/.local/share/ji/ji.identity.age`. Does not create manifest.toml — use `ji add` for that.

## ji add

Add files to the tracking manifest.

```
ji add <PATH>... [--include <PATTERN>] [--exclude <PATTERN>]
```

| Option | Description |
|---|---|
| `--include <PATTERN>` | Only include files matching this glob |
| `--exclude <PATTERN>` | Exclude files matching this glob |

Paths are relative to `$HOME`. Directories are recursed automatically. `.jiignore` rules and default exclusions (`.ssh/`, `.DS_Store`, `node_modules/`) are applied.

## ji rm

Remove files from the manifest (does not delete actual files).

```
ji rm <PATH>... [--all]
```

| Option | Description |
|---|---|
| `--all` | Remove all tracked files |

## ji list

List tracked files and their checksums.

```
ji list [--json]
```

| Option | Description |
|---|---|
| `--json` | Output as JSON |

## ji status

Compare local files against manifest checksums.

```
ji status [-s|--short]
```

| Option | Description |
|---|---|
| `-s`, `--short` | Compact output (path + marker only) |

Status markers: `M` (modified), `-` (deleted), `A` (in manifest but missing on disk), ` ` (unchanged).

## ji diff

Show unified diff of changed files. Requires a prior `ji pack` to populate the cache.

```
ji diff [<PATH>]
```

Without `<PATH>`, diffs all changed files. Binary files are reported as "binary files differ".

## ji pack

Pack tracked files into an encrypted `.ji` archive.

```
ji pack [-o <PATH>] [--strict] [--verbose]
```

| Option | Description |
|---|---|
| `-o <PATH>` | Output path (default: `~/.local/share/ji/<hostname>.ji`) |
| `--strict` | Refuse to pack if any checksum mismatches |
| `--verbose` | Show diff of changed files |

Updates the cache in `~/.local/share/ji/cache/` for subsequent `ji diff`.

## ji unpack

Decrypt a `.ji` archive and restore files to `$HOME`.

```
ji unpack <INPUT.ji> [--dry-run] [--force] [--interactive] [--backup]
```

| Option | Description |
|---|---|
| `--dry-run` | Show what would happen without writing |
| `--force` | Overwrite existing files |
| `--interactive` | Ask before overwriting each file |
| `--backup` | Rename existing files to `.bak` before overwriting |

Default behavior: skip files that already exist.

## ji check

Verify `.ji` file integrity.

```
ji check <INPUT.ji> [--deep]
```

| Option | Description |
|---|---|
| `--deep` | Full verification: decrypt and check per-file checksums |

Fast mode (default) verifies magic bytes, version, cipher type, and HMAC — no decryption required. Deep mode decrypts and verifies every file against the manifest checksums.

## ji doctor

Diagnose configuration, keys, and connectivity.

```
ji doctor [--full] [--json]
```

| Option | Description |
|---|---|
| `--full` | Include remote connectivity and archive scan |
| `--json` | Output as JSON |

Exit code 0 when all checks pass, 1 when errors found. See `docs/troubleshooting.md` for interpreting results.

## ji remote

Manage remote endpoints.

```
ji remote add <NAME> --type webdav --url <URL> [--user <USER>]
ji remote remove <NAME>
ji remote list [--json]
ji remote test <NAME>
ji remote files <NAME>
ji remote delete <NAME> <FILE>
```

Passwords are prompted interactively and never stored.

## ji push / pull / sync

Transfer `.ji` files to and from remote endpoints.

```
ji push <NAME> <INPUT.ji>
ji pull <NAME>
ji sync <NAME>
```

`sync` detects direction automatically: if local has changes, it packs and pushes; otherwise it pulls. Always run `ji pack` before `ji push`.

## ji recipient

Manage recipients (who can decrypt) of a `.ji` file.

```
ji recipient list <INPUT.ji>
ji recipient add --key <PUBKEY> <INPUT.ji>
ji recipient remove --key <PUBKEY> <INPUT.ji>
```

Adding or removing a recipient requires the ability to decrypt the file (you must hold at least one existing private key).

## ji sync

Bidirectional sync with a remote. Detects direction automatically.

```
ji sync <NAME>
```

If local manifest has changes, packs and pushes. Otherwise pulls from the remote.

## ji completion

Generate shell completion scripts.

```
ji completion <SHELL>
```

Supported shells: `bash`, `zsh`, `fish`.

```bash
ji completion fish > ~/.config/fish/completions/ji.fish
```
