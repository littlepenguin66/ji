use crate::archive::format;
use crate::crypto::age::AgeCipher;
use crate::crypto::Cipher;
use crate::error::Result;
use std::io::{Read, Write};
use std::path::PathBuf;

pub fn run_list(input: PathBuf) -> Result<()> {
    let data = std::fs::read(&input)
        .map_err(|e| crate::error::Error::Archive(format!("read: {e}")))?;

    // Read header to get index_len
    let mut cursor = std::io::Cursor::new(&data);
    let (_cipher, index_len) = format::read_header(&mut cursor)?;

    // Skip index + HMAC to get to encrypted payload
    let payload_start = format::HEADER_SIZE as u64 + index_len as u64;
    let encrypted = &data[payload_start as usize..];

    let recipients = AgeCipher::list_recipients(encrypted)?;

    if recipients.is_empty() {
        println!("(no recipients found)");
    } else {
        for r in &recipients {
            println!("{r}");
        }
    }

    Ok(())
}

pub fn run_add(key: String, input: PathBuf) -> Result<()> {
    let data = std::fs::read(&input)
        .map_err(|e| crate::error::Error::Archive(format!("read: {e}")))?;

    // Read header
    let mut cursor = std::io::Cursor::new(&data);
    let (_cipher, index_len) = format::read_header(&mut cursor)?;

    // Get index
    let mut index_buf = vec![0u8; index_len as usize];
    cursor
        .read_exact(&mut index_buf)
        .map_err(|e| crate::error::Error::Archive(format!("read index: {e}")))?;

    // Verify HMAC before proceeding
    let _idx = format::read_index(&mut std::io::Cursor::new(&index_buf))?;

    // Decrypt payload
    let payload_start = format::HEADER_SIZE as u64 + index_len as u64;
    let encrypted = &data[payload_start as usize..];
    let decrypted = AgeCipher::decrypt(encrypted)?;

    // Get existing recipients and add new one
    let existing = AgeCipher::list_recipients(encrypted)?;
    let mut new_recipients: Vec<String> = existing
        .into_iter()
        .filter(|r| r.starts_with("X25519 "))
        .collect();
    new_recipients.push(key);

    // Re-encrypt
    let re_encrypted = AgeCipher::encrypt(&decrypted, &new_recipients)?;

    // Re-write file atomically
    let tmp = input.with_extension("ji_tmp");
    {
        let mut file = std::fs::File::create(&tmp)
            .map_err(|e| crate::error::Error::Archive(format!("create tmp: {e}")))?;
        file.write_all(&data[..format::HEADER_SIZE])
            .map_err(|e| crate::error::Error::Archive(format!("write header: {e}")))?;
        file.write_all(&index_buf)
            .map_err(|e| crate::error::Error::Archive(format!("write index: {e}")))?;
        file.write_all(&re_encrypted)
            .map_err(|e| crate::error::Error::Archive(format!("write payload: {e}")))?;
        file.sync_all()
            .map_err(|e| crate::error::Error::Archive(format!("fsync: {e}")))?;
    }
    std::fs::rename(&tmp, &input)
        .map_err(|e| crate::error::Error::Archive(format!("rename: {e}")))?;

    println!("Recipient added.");
    Ok(())
}

pub fn run_remove(key: String, input: PathBuf) -> Result<()> {
    let data = std::fs::read(&input)
        .map_err(|e| crate::error::Error::Archive(format!("read: {e}")))?;

    let mut cursor = std::io::Cursor::new(&data);
    let (_cipher, index_len) = format::read_header(&mut cursor)?;

    // Read and verify index HMAC
    let mut index_buf = vec![0u8; index_len as usize];
    cursor
        .read_exact(&mut index_buf)
        .map_err(|e| crate::error::Error::Archive(format!("read index: {e}")))?;
    format::read_index(&mut std::io::Cursor::new(&index_buf))?;

    let payload_start = format::HEADER_SIZE as u64 + index_len as u64;
    let encrypted = &data[payload_start as usize..];

    let decrypted = AgeCipher::decrypt(encrypted)?;
    let existing = AgeCipher::list_recipients(encrypted)?;

    let new_recipients: Vec<String> = existing
        .into_iter()
        .filter(|r| r.starts_with("X25519 ") && !r.contains(&key))
        .collect();

    if new_recipients.is_empty() {
        return Err(crate::error::Error::Crypto(
            "cannot remove last recipient".into(),
        ));
    }

    let re_encrypted = AgeCipher::encrypt(&decrypted, &new_recipients)?;
    let tmp = input.with_extension("ji_tmp");
    let mut file = std::fs::File::create(&tmp)
        .map_err(|e| crate::error::Error::Archive(format!("create tmp: {e}")))?;
    file.write_all(&data[..payload_start as usize])
        .map_err(|e| crate::error::Error::Archive(format!("write header: {e}")))?;
    file.write_all(&re_encrypted)
        .map_err(|e| crate::error::Error::Archive(format!("write payload: {e}")))?;
    file.sync_all()
        .map_err(|e| crate::error::Error::Archive(format!("fsync: {e}")))?;
    std::fs::rename(&tmp, &input)
        .map_err(|e| crate::error::Error::Archive(format!("rename: {e}")))?;

    println!("Recipient removed.");
    Ok(())
}
