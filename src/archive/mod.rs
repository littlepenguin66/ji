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

    let tar_data = build_tar(manifest)?;

    let index = build_index(manifest)?;

    let compressed = zstd::encode_all(&tar_data[..], 3)
        .map_err(|e| Error::Archive(format!("zstd compress: {e}")))?;

    let encrypted = encrypt_with(cipher, &compressed, recipients)?;

    let mut index_buf = Vec::new();
    format::write_index(&mut index_buf, &index)?;
    let index_len = index_buf.len() as u32;

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

pub fn unpack_archive(
    input: &Path,
    dry_run: bool,
    force: bool,
    interactive: bool,
    backup: bool,
) -> Result<usize> {
    let mut file = std::fs::File::open(input)
        .map_err(|e| Error::Archive(format!("open: {e}")))?;

    let (cipher, index_len) = format::read_header(&mut file)?;

    let mut index_buf = vec![0u8; index_len as usize];
    file.read_exact(&mut index_buf)
        .map_err(|e| Error::Archive(format!("read index: {e}")))?;
    let _index = format::read_index(&mut Cursor::new(&index_buf))?;

    let mut encrypted = Vec::new();
    file.read_to_end(&mut encrypted)
        .map_err(|e| Error::Archive(format!("read payload: {e}")))?;

    let decrypted = decrypt_with(cipher, &encrypted)?;

    let tar_data = zstd::decode_all(&decrypted[..])
        .map_err(|e| Error::Archive(format!("zstd decompress: {e}")))?;

    let home = manifest::resolve_home("");
    let restored = extract_tar(&tar_data, &home, dry_run, force, interactive, backup)?;

    Ok(restored)
}

fn build_tar(manifest: &Manifest) -> Result<Vec<u8>> {
    let mut archive = tar::Builder::new(Vec::new());

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

        if path_str == ".ji_manifest.toml" {
            continue;
        }

        let Some(rel_path) = path_str.strip_prefix("files/") else {
            continue;
        };
        let rel_path = rel_path.to_string();

        if rel_path.split(std::path::MAIN_SEPARATOR).any(|c| c == "..") {
            return Err(Error::Archive(format!(
                "path traversal denied: {rel_path}"
            )));
        }

        let dest = home.join(&rel_path);

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


pub fn verify_archive(input: &Path, deep: bool) -> Result<()> {
    let mut file = std::fs::File::open(input)
        .map_err(|e| Error::Archive(format!("open: {e}")))?;

    let (cipher, index_len) = format::read_header(&mut file)?;
    let cipher_name = match cipher {
        CipherType::Age => "age",
        CipherType::Pgp => "pgp",
    };
    println!("cipher: {cipher_name}");

    let mut index_buf = vec![0u8; index_len as usize];
    file.read_exact(&mut index_buf)
        .map_err(|e| Error::Archive(format!("read index: {e}")))?;
    let index = format::read_index(&mut std::io::Cursor::new(&index_buf))?;

    println!("files:");
    for entry in &index.entries {
        println!("  {} ({} bytes)", entry.name, entry.size);
    }
    println!("total: {} bytes", index.total_size);
    println!("HMAC: OK");

    if deep {
        let mut encrypted = Vec::new();
        file.read_to_end(&mut encrypted)
            .map_err(|e| Error::Archive(format!("read payload: {e}")))?;

        let decrypted = decrypt_with(cipher, &encrypted)?;
        let tar_data = zstd::decode_all(&decrypted[..])
            .map_err(|e| Error::Archive(format!("decompress: {e}")))?;

        verify_manifest_checksums(&tar_data)?;
        println!("deep check: OK");
    }

    println!("ji: {} checksum OK", input.display());
    Ok(())
}

fn verify_manifest_checksums(tar_data: &[u8]) -> Result<()> {
    let mut archive = tar::Archive::new(Cursor::new(tar_data));

    let manifest_toml = {
        let mut manifest_buf = Vec::new();
        for entry in archive
            .entries()
            .map_err(|e| Error::Archive(format!("tar entries: {e}")))?
        {
            let mut entry = entry.map_err(|e| Error::Archive(format!("tar entry: {e}")))?;
            let path = entry.path()
                .map_err(|e| Error::Archive(format!("tar path: {e}")))?;
            if path.to_string_lossy() == ".ji_manifest.toml" {
                entry.read_to_end(&mut manifest_buf)
                    .map_err(|e| Error::Archive(format!("read manifest: {e}")))?;
                break;
            }
        }
        manifest_buf
    };

    if manifest_toml.is_empty() {
        return Err(Error::Archive(".ji_manifest.toml not found in archive".into()));
    }

    let manifest: Manifest = toml::from_str(&String::from_utf8_lossy(&manifest_toml))
        .map_err(|e| Error::Archive(format!("manifest parse: {e}")))?;

    let mut archive = tar::Archive::new(Cursor::new(tar_data));
    for entry in archive
        .entries()
        .map_err(|e| Error::Archive(format!("tar entries: {e}")))?
    {
        let mut entry = entry.map_err(|e| Error::Archive(format!("tar entry: {e}")))?;
        let path_str = {
            let p = entry.path()
                .map_err(|e| Error::Archive(format!("tar path: {e}")))?;
            p.to_string_lossy().to_string()
        };

        if let Some(rel) = path_str.strip_prefix("files/") {
            if let Some(expected) = manifest.get(rel) {
                let mut content = Vec::new();
                entry
                    .read_to_end(&mut content)
                    .map_err(|e| Error::Archive(format!("read: {e}")))?;
                let actual = manifest::compute_checksum_reader(&content);
                if actual != expected.checksum {
                    println!("CHECKSUM MISMATCH: {rel}");
                    println!("  expected: {}", expected.checksum);
                    println!("  got:      {actual}");
                } else {
                    println!("  OK: {rel}");
                }
            }
        }
    }

    Ok(())
}

pub fn list_archive_recipients(input: &Path) -> Result<Vec<String>> {
    let data = std::fs::read(input)
        .map_err(|e| Error::Archive(format!("read: {e}")))?;

    let mut cursor = Cursor::new(&data);
    let (cipher, index_len) = format::read_header(&mut cursor)?;

    let payload_start = format::HEADER_SIZE as usize + index_len as usize;
    let encrypted = &data[payload_start..];

    decrypt_with_recipients(cipher, encrypted)
}

pub fn add_archive_recipient(input: &Path, key: &str) -> Result<()> {
    let data = std::fs::read(input)
        .map_err(|e| Error::Archive(format!("read: {e}")))?;

    let mut cursor = Cursor::new(&data);
    let (cipher, index_len) = format::read_header(&mut cursor)?;

    let header_end = format::HEADER_SIZE;
    let payload_start = header_end as usize + index_len as usize;
    let index_buf = &data[header_end..payload_start];

    format::read_index(&mut Cursor::new(index_buf))?;

    let encrypted = &data[payload_start..];
    let existing = decrypt_with_recipients(cipher, encrypted)?;
    let decrypted = decrypt_with(cipher, encrypted)?;

    let mut new_recipients: Vec<String> = existing
        .into_iter()
        .filter(|r| r.starts_with("X25519 "))
        .collect();
    new_recipients.push(key.to_string());

    let re_encrypted = encrypt_with(cipher, &decrypted, &new_recipients)?;

    rewrite_payload(input, &data, cipher, index_buf, &re_encrypted)
}

pub fn remove_archive_recipient(input: &Path, key: &str) -> Result<()> {
    let data = std::fs::read(input)
        .map_err(|e| Error::Archive(format!("read: {e}")))?;

    let mut cursor = Cursor::new(&data);
    let (cipher, index_len) = format::read_header(&mut cursor)?;

    let header_end = format::HEADER_SIZE;
    let payload_start = header_end as usize + index_len as usize;
    let index_buf = &data[header_end..payload_start];

    format::read_index(&mut Cursor::new(index_buf))?;

    let encrypted = &data[payload_start..];
    let existing = decrypt_with_recipients(cipher, encrypted)?;
    let decrypted = decrypt_with(cipher, encrypted)?;

    let new_recipients: Vec<String> = existing
        .into_iter()
        .filter(|r| r.starts_with("X25519 ") && !r.contains(key))
        .collect();

    if new_recipients.is_empty() {
        return Err(Error::Crypto("cannot remove last recipient".into()));
    }

    let re_encrypted = encrypt_with(cipher, &decrypted, &new_recipients)?;

    rewrite_payload(input, &data, cipher, index_buf, &re_encrypted)
}

fn decrypt_with_recipients(cipher: CipherType, encrypted: &[u8]) -> Result<Vec<String>> {
    match cipher {
        CipherType::Age => AgeCipher::list_recipients(encrypted),
        CipherType::Pgp => Err(Error::Crypto(
            "PGP recipient listing not supported".into(),
        )),
    }
}

fn rewrite_payload(
    input: &Path,
    header: &[u8],
    _cipher: CipherType,
    index_buf: &[u8],
    new_payload: &[u8],
) -> Result<()> {
    let tmp = input.with_extension("ji_tmp");
    {
        let mut file = std::fs::File::create(&tmp)
            .map_err(|e| Error::Archive(format!("create tmp: {e}")))?;
        file.write_all(&header[..format::HEADER_SIZE])
            .map_err(|e| Error::Archive(format!("write header: {e}")))?;
        file.write_all(index_buf)
            .map_err(|e| Error::Archive(format!("write index: {e}")))?;
        file.write_all(new_payload)
            .map_err(|e| Error::Archive(format!("write payload: {e}")))?;
        file.sync_all()
            .map_err(|e| Error::Archive(format!("fsync: {e}")))?;
    }
    std::fs::rename(&tmp, input)
        .map_err(|e| Error::Archive(format!("rename: {e}")))?;
    Ok(())
}
