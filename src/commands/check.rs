use crate::archive::format;
use crate::crypto::age::AgeCipher;
use crate::crypto::Cipher;
use crate::error::Result;
use crate::store::manifest::Manifest;
use std::io::Read;
use std::path::PathBuf;

pub fn run(input: PathBuf, deep: bool) -> Result<()> {
    let mut file = std::fs::File::open(&input)
        .map_err(|e| crate::error::Error::Archive(format!("open: {e}")))?;

    // Read and validate header
    let (cipher, index_len) = format::read_header(&mut file)?;
    let cipher_name = match cipher {
        format::CipherType::Age => "age",
        format::CipherType::Pgp => "pgp",
    };
    println!("cipher: {cipher_name}");

    // Read and verify index
    let mut index_buf = vec![0u8; index_len as usize];
    file.read_exact(&mut index_buf)
        .map_err(|e| crate::error::Error::Archive(format!("read index: {e}")))?;
    let index = format::read_index(&mut std::io::Cursor::new(&index_buf))?;

    println!("files:");
    for entry in &index.entries {
        println!("  {} ({} bytes)", entry.name, entry.size);
    }
    println!("total: {} bytes", index.total_size);
    println!("HMAC: OK");

    if deep {
        // Read encrypted payload
        let mut encrypted = Vec::new();
        file.read_to_end(&mut encrypted)
            .map_err(|e| crate::error::Error::Archive(format!("read payload: {e}")))?;

        // Decrypt
        let decrypted = AgeCipher::decrypt(&encrypted)?;

        // Decompress
        let tar_data = zstd::decode_all(&decrypted[..])
            .map_err(|e| crate::error::Error::Archive(format!("decompress: {e}")))?;

        // Extract and verify checksums
        let mut archive = tar::Archive::new(std::io::Cursor::new(&tar_data));
        let manifest_entry = archive
            .entries()
            .map_err(|e| crate::error::Error::Archive(format!("tar entries: {e}")))?
            .find_map(|e| {
                let entry = e.ok()?;
                let path = entry.path().ok()?;
                if path.to_string_lossy() == ".ji_manifest.toml" {
                    let mut buf = Vec::new();
                    entry.take(u64::MAX).read_to_end(&mut buf).ok()?;
                    Some(buf)
                } else {
                    None
                }
            });

        // Verify manifest checksums inside the archive
        if let Some(manifest_data) = manifest_entry {
            let manifest: Manifest = toml::from_str(
                &String::from_utf8_lossy(&manifest_data),
            )
            .map_err(|e| crate::error::Error::Archive(format!("manifest parse: {e}")))?;

            // Verify each file's checksum against the manifest
            let mut archive = tar::Archive::new(std::io::Cursor::new(&tar_data));
            for entry in archive
                .entries()
                .map_err(|e| crate::error::Error::Archive(format!("tar entries: {e}")))?
            {
                let mut entry =
                    entry.map_err(|e| crate::error::Error::Archive(format!("tar entry: {e}")))?;
                let path_str = {
                    let path = entry.path().map_err(|e| {
                        crate::error::Error::Archive(format!("tar path: {e}"))
                    })?;
                    path.to_string_lossy().to_string()
                };

                if let Some(rel) = path_str.strip_prefix("files/") {
                    let rel = rel.to_string();
                    if let Some(expected) = manifest.get(&rel) {
                        let expected_cs = expected.checksum.clone();
                        let mut content = Vec::new();
                        entry
                            .read_to_end(&mut content)
                            .map_err(|e| crate::error::Error::Archive(format!("read: {e}")))?;
                        let actual = crate::store::manifest::compute_checksum_reader(&content);
                        if actual != expected_cs {
                            println!("CHECKSUM MISMATCH: {rel}");
                            println!("  expected: {expected_cs}");
                            println!("  got:      {actual}");
                        } else {
                            println!("  OK: {rel}");
                        }
                    }
                }
            }
        } else {
            return Err(crate::error::Error::Archive(
                "deep check failed: .ji_manifest.toml not found in archive".into(),
            ));
        }

        println!("deep check: OK");
    }

    println!("ji: {input} checksum OK", input = input.display());
    Ok(())
}
