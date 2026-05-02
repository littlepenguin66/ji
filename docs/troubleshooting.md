# Troubleshooting

Each section maps to a `ji doctor` check category. Run `ji doctor` first to identify the problem, then follow the corresponding guide.

## Config

### `config.toml not found`

**doctor**: `✗ config.toml not found`

**Cause**: `ji init` has not been run, or the config directory was deleted.

**Fix**:
```bash
ji init                    # interactive
ji init -a                 # auto-generate keypair
ji init --key ~/.ssh/id_ed25519.pub  # use existing SSH key
```

### `no recipients configured`

**doctor**: `✗ no recipients configured`

**Cause**: config.toml exists but has an empty `recipients` list.

**Fix**:
```bash
ji init --force --key ~/.ssh/id_ed25519.pub
```

### `failed to parse config.toml`

**doctor**: `✗ failed to parse config.toml`

**Cause**: config.toml is malformed TOML.

**Fix**: Inspect and repair `~/.config/ji/config.toml`, or regenerate:
```bash
ji init --force
```

## Keys

### `no ji identity key`

**doctor**: `! no ji identity key`

**Cause**: No age keypair was generated (e.g. `ji init` was run with `--key` but never generated its own identity).

**Fix**:
```bash
ji init -a --force
```

### `no private key available for decryption`

**doctor**: none (only surfaces at decrypt time)

**Cause**: The `.ji` file was encrypted for a key you don't have. Your identity key may be missing, or the file was encrypted for a different machine.

**Fix**:
```bash
# Check if you have the identity key
ls ~/.local/share/ji/ji.identity.age

# Check what keys the .ji file needs
ji recipient list my.ji

# Regenerate identity and add yourself
ji init -a --force
ji recipient add --key ~/.local/share/ji/ji.identity.age.pub my.ji
```

### `ssh-agent not running`

**doctor**: `! ssh-agent not running`

**Cause**: SSH agent is not active. age will fall back to reading key files directly from `~/.ssh/`.

**Fix** (optional):
```bash
eval $(ssh-agent) && ssh-add
```

## Manifest

### `manifest.toml not found`

**doctor**: `! manifest.toml not found`

**Cause**: No files have been added yet.

**Fix**:
```bash
ji add .zshrc .gitconfig
```

### `no files tracked`

**doctor**: `! no files tracked`

**Cause**: manifest.toml exists but contains no entries.

**Fix**: `ji add <file>...`

### `N file(s) missing on disk`

**doctor**: `! N file(s) missing on disk`

**Cause**: Some tracked files have been moved or deleted since they were added.

**Fix**: Re-add the missing files, or remove them from the manifest:
```bash
ji status          # see which files are missing
ji rm <file>       # remove from manifest
ji add <file>      # re-add if moved to new location
```

### `N file(s) modified since last pack`

**doctor**: `! N file(s) modified since last pack`

**Cause**: Tracked files have been edited since the last `ji pack`. This is normal.

**Fix**:
```bash
ji status          # see which files changed
ji pack            # update the .ji file
```

### `checksum mismatch`

**doctor**: none (surfaces during `ji pack --strict` or `ji check --deep`)

**Cause**: A file's content differs from its recorded checksum in the manifest.

**Fix**:
```bash
ji status          # identify the mismatched files
ji pack            # re-pack to update checksums
```

## Archive

### `invalid magic bytes`

**doctor**: `✗ invalid .ji file`

**Cause**: The file is not a `.ji` archive, or is corrupted at the header level.

**Fix**: Verify you have the correct file. Re-download or re-pack.

### `unsupported version`

**doctor**: `✗ invalid .ji file`

**Cause**: The `.ji` file was created by a newer version of ji.

**Fix**: Upgrade ji to the latest version.

### `HMAC verification failed`

**doctor**: `✗ HMAC verification failed`

**Cause**: The `.ji` file's plaintext index has been tampered with or corrupted. This could indicate a damaged file (bad transfer, disk error) or malicious modification.

**Fix**: Re-download from the remote, or re-pack from the source machine. If the file was transferred manually, verify the transfer completed successfully (check file sizes match).

## Remote

### 401 Unauthorized (WebDAV)

**doctor**: `✗ WebDAV unreachable`

**Cause**: Wrong username or password.

**Fix**: Verify credentials and try again. Check the remote URL is correct:
```bash
ji remote test nas
```

### Connection refused

**doctor**: `✗ WebDAV unreachable` or `✗ SSH unreachable`

**Cause**: The remote server is unavailable, the URL is wrong, or a firewall is blocking the connection.

**Fix**:
```bash
# Verify the URL
ji remote list

# Test connectivity
ji remote test nas

# For WebDAV: try with curl
curl -I https://nas.local/ji/

# For SSH: try direct connection
ssh user@host
```

### `SSH unreachable`

**doctor**: `✗ SSH unreachable`

**Cause**: SSH connection failed — wrong host, port, or key configuration.

**Fix**:
```bash
# Verify key-based access works
ssh user@host echo ok

# Set up SSH key if needed
ssh-copy-id user@host
```

### Push failed but connection works

**Cause**: The remote directory doesn't exist, or you lack write permissions.

**Fix**: Create the target directory on the remote server and ensure correct permissions. For WebDAV, ensure the server allows PUT requests.
