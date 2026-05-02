pub mod format;

use crate::crypto::age::AgeCipher;
use crate::crypto::Cipher;
use crate::error::{Error, Result};
use crate::store::manifest::{self, Manifest};
use format::{CipherType, IndexEntry, PlainIndex};
use std::io::{Cursor, Read, Write};
use std::path::Path;

fn encrypt_with(cipher: CipherType, data: &[u8], recipients: &[String]) -> Result<Vec<u8>> {
    match cipher {
        CipherType::Age => AgeCipher::encrypt(data, recipients),
        CipherType::Pgp => {
            #[cfg(feature = "pgp")]
            {
                crate::crypto::pgp::PgpCipher::encrypt(data, recipients)
            }
            #[cfg(not(feature = "pgp"))]
            {
                Err(Error::Crypto(
                    "PGP support not compiled. Rebuild with --features pgp".into(),
                ))
            }
        }
    }
}

fn decrypt_with(cipher: CipherType, data: &[u8]) -> Result<Vec<u8>> {
    match cipher {
        CipherType::Age => AgeCipher::decrypt(data),
        CipherType::Pgp => {
            #[cfg(feature = "pgp")]
            {
                crate::crypto::pgp::PgpCipher::decrypt(data)
            }
            #[cfg(not(feature = "pgp"))]
            {
                Err(Error::Crypto(
                    "PGP support not compiled. Rebuild with --features pgp".into(),
                ))
            }
        }
    }
}

/// Pack manifest-tracked files into a .ji archive, written atomically to `output`.
pub fn pack_archive(
    output: &Path,
    manifest: &Manifest,
    recipients: &[String],
    cipher: CipherType,
) -> Result<()> {
    if manifest.files.is_empty() {
        return Err(Error::Archive("no files to pack".into()));
    }

    if recipients.is_empty() {
        return Err(Error::Archive("no recipients configured".into()));
    }

    // Build tar in memory
    let tar_data = build_tar(manifest)?;

    // Build index
    let index = build_index(manifest)?;

    // Compress with zstd
    let compressed = zstd::encode_all(&tar_data[..], 3)
        .map_err(|e| Error::Archive(format!("zstd compress: {e}")))?;

    // Encrypt
    let encrypted = encrypt_with(cipher, &compressed, recipients)?;

    // Serialize index to bytes
    let mut index_buf = Vec::new();
    format::write_index(&mut index_buf, &index)?;
    let index_len = index_buf.len() as u32;

    // Atomic write: tmp file → fsync → rename
    let tmp_path = output.with_extension("ji_tmp");
    {
        let mut file = std::fs::File::create(&tmp_path)
            .map_err(|e| Error::Archive(format!("create tmp: {e}")))?;

        format::write_header(&mut file, cipher, index_len)?;
        file.write_all(&index_buf)
            .map_err(|e| Error::Archive(format!("write index: {e}")))?;
        file.write_all(&encrypted)
            .map_err(|e| Error::Archive(format!("write payload: {e}")))?;
        file.sync_all()
            .map_err(|e| Error::Archive(format!("fsync: {e}")))?;
    }

    std::fs::rename(&tmp_path, output)
        .map_err(|e| Error::Archive(format!("rename: {e}")))?;

    Ok(())
}

/// Unpack a .ji archive, restoring files to `$HOME`.
/// Returns the number of restored files.
pub fn unpack_archive(
    input: &Path,
    dry_run: bool,
    force: bool,
    interactive: bool,
    backup: bool,
) -> Result<usize> {
    let mut file = std::fs::File::open(input)
        .map_err(|e| Error::Archive(format!("open: {e}")))?;

    // Read header
    let (cipher, index_len) = format::read_header(&mut file)?;

    // Read index
    let mut index_buf = vec![0u8; index_len as usize];
    file.read_exact(&mut index_buf)
        .map_err(|e| Error::Archive(format!("read index: {e}")))?;
    let _index = format::read_index(&mut Cursor::new(&index_buf))?;

    // Read encrypted payload
    let mut encrypted = Vec::new();
    file.read_to_end(&mut encrypted)
        .map_err(|e| Error::Archive(format!("read payload: {e}")))?;

    // Decrypt
    let decrypted = decrypt_with(cipher, &encrypted)?;

    // Decompress
    let tar_data = zstd::decode_all(&decrypted[..])
        .map_err(|e| Error::Archive(format!("zstd decompress: {e}")))?;

    // Extract tar
    let home = manifest::resolve_home("");
    let restored = extract_tar(&tar_data, &home, dry_run, force, interactive, backup)?;

    Ok(restored)
}

fn build_tar(manifest: &Manifest) -> Result<Vec<u8>> {
    let mut archive = tar::Builder::new(Vec::new());

    // Add manifest to the archive
    let manifest_toml = toml::to_string_pretty(manifest)?;
    let mut header = tar::Header::new_gnu();
    header
        .set_path(".ji_manifest.toml")
        .map_err(|e| Error::Archive(format!("tar header: {e}")))?;
    header.set_size(manifest_toml.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    archive
        .append(&header, Cursor::new(manifest_toml.as_bytes()))
        .map_err(|e| Error::Archive(format!("tar append manifest: {e}")))?;

    // Add each tracked file
    for rel_path in manifest.list_paths() {
        let abs_path = manifest::resolve_home(rel_path);
        if !abs_path.exists() {
            return Err(Error::Archive(format!("file not found: {rel_path}")));
        }

        let content = std::fs::read(&abs_path)
            .map_err(|e| Error::Archive(format!("read {rel_path}: {e}")))?;

        let archive_path = format!("files/{rel_path}");
        let mut header = tar::Header::new_gnu();
        header
            .set_path(&archive_path)
            .map_err(|e| Error::Archive(format!("tar header {rel_path}: {e}")))?;
        header.set_size(content.len() as u64);

        // Preserve original mode if possible
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if let Ok(meta) = std::fs::metadata(&abs_path) {
                header.set_mode(meta.mode());
            } else {
                header.set_mode(0o644);
            }
        }
        #[cfg(not(unix))]
        {
            header.set_mode(0o644);
        }

        header.set_cksum();
        archive
            .append(&header, Cursor::new(&content))
            .map_err(|e| Error::Archive(format!("tar append {rel_path}: {e}")))?;
    }

    archive
        .into_inner()
        .map_err(|e| Error::Archive(format!("tar finish: {e}")))
}

fn build_index(manifest: &Manifest) -> Result<PlainIndex> {
    let mut entries = Vec::new();
    let mut total_size = 0u64;

    for rel_path in manifest.list_paths() {
        let abs_path = manifest::resolve_home(rel_path);
        let size = if abs_path.exists() {
            let meta = std::fs::metadata(&abs_path)
                .map_err(|e| Error::Archive(format!("stat {rel_path}: {e}")))?;
            meta.len()
        } else {
            return Err(Error::Archive(format!("file not found: {rel_path}")));
        };
        total_size = total_size.saturating_add(size);
        entries.push(IndexEntry {
            name: rel_path.clone(),
            size,
        });
    }

    Ok(PlainIndex {
        entries,
        total_size,
    })
}

fn extract_tar(
    tar_data: &[u8],
    home: &Path,
    dry_run: bool,
    force: bool,
    interactive: bool,
    backup: bool,
) -> Result<usize> {
    let mut archive = tar::Archive::new(Cursor::new(tar_data));
    let mut restored = 0usize;

    for entry_result in archive
        .entries()
        .map_err(|e| Error::Archive(format!("tar entries: {e}")))?
    {
        let mut entry =
            entry_result.map_err(|e| Error::Archive(format!("tar entry: {e}")))?;

        let (path_str, mode) = {
            let path = entry
                .path()
                .map_err(|e| Error::Archive(format!("tar path: {e}")))?;
            let mode = entry.header().mode().unwrap_or(0o644);
            (path.to_string_lossy().to_string(), mode)
        };

        // Skip the manifest file
        if path_str == ".ji_manifest.toml" {
            continue;
        }

        // Extract files/ prefix
        let Some(rel_path) = path_str.strip_prefix("files/") else {
            continue;
        };
        let rel_path = rel_path.to_string();

        // Path traversal protection
        if rel_path.split(std::path::MAIN_SEPARATOR).any(|c| c == "..") {
            return Err(Error::Archive(format!(
                "path traversal denied: {rel_path}"
            )));
        }

        let dest = home.join(&rel_path);

        // Conflict resolution
        if dest.exists() {
            if dry_run {
                println!("would overwrite: {rel_path}");
                restored += 1;
                continue;
            }
            if interactive {
                eprintln!("ji: {rel_path} already exists. Overwrite? [y/N]");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).ok();
                if !input.trim().eq_ignore_ascii_case("y") {
                    println!("skipped: {rel_path}");
                    continue;
                }
            } else if backup {
                let backup_path = dest.with_extension("bak");
                std::fs::rename(&dest, &backup_path)
                    .map_err(|e| Error::Archive(format!("backup {rel_path}: {e}")))?;
                println!("backed up: {rel_path} -> {rel_path}.bak");
            } else if !force {
                println!("skipped: {rel_path} (exists)");
                continue;
            }
        }

        if dry_run {
            println!("would restore: {rel_path}");
            restored += 1;
            continue;
        }

        // Atomic write
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| Error::Archive(format!("mkdir {rel_path}: {e}")))?;
        }

        let tmp = dest.with_extension("ji_tmp");
        let mut file = std::fs::File::create(&tmp)
            .map_err(|e| Error::Archive(format!("create {rel_path}: {e}")))?;

        let mut content = Vec::new();
        entry
            .read_to_end(&mut content)
            .map_err(|e| Error::Archive(format!("read entry {rel_path}: {e}")))?;
        file.write_all(&content)
            .map_err(|e| Error::Archive(format!("write {rel_path}: {e}")))?;
        file.sync_all()
            .map_err(|e| Error::Archive(format!("fsync {rel_path}: {e}")))?;

        // Restore permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Err(e) =
                std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(mode))
            {
                eprintln!("ji: warning: could not set permissions on {rel_path}: {e}");
            }
        }

        std::fs::rename(&tmp, &dest)
            .map_err(|e| Error::Archive(format!("rename {rel_path}: {e}")))?;
        println!("restored: {rel_path}");
        restored += 1;
    }

    Ok(restored)
}
