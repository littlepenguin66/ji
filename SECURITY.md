# Security

## Threat Model

ji protects dotfiles in transit and at rest on remote storage. It does NOT protect against:

| Threat | Protected? |
|---|---|
| `.ji` file intercepted in transit | Yes — age/PGP encryption |
| `.ji` file tampered with | Yes — HMAC-SHA256 index signature |
| `.ji` file metadata (file names, sizes) leaked | No — plaintext index is intentional for `ji list` without decryption |
| Local attacker with disk access reading your dotfiles | No — dotfiles live unencrypted in `$HOME` by design |
| Private key compromise | No — rotate keys with `ji recipient add/remove` |

The plaintext index stores file names and sizes only — never file contents. If you have files with sensitive names (e.g. `.env.production`), be aware they are visible to anyone with the `.ji` file.

## HMAC Key

The index HMAC key is a hardcoded application secret (`b"ji-dotfiles-hmac-key-v1"`). This provides **integrity verification only** — it proves the index was created by ji and has not been modified. It is not a cryptographic authentication mechanism and does not provide confidentiality.

## Reporting a Vulnerability

Email `security@ji.example.com` or open a private vulnerability report on GitHub. Please include:

- Affected version
- Steps to reproduce
- Impact assessment

## Supply Chain

```bash
cargo audit     # check dependencies for known vulnerabilities
```

Run before each release. Dependencies are pinned in `Cargo.lock`.

## Design

See [docs/architecture.md](docs/architecture.md) for the full security-relevant design details: encryption flow, atomic write pattern, key discovery, and `.ji` binary format.
