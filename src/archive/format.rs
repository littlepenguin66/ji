use crate::error::{Error, Result};
use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;
use std::io::{Cursor, Read, Write};

pub const MAGIC: [u8; 3] = [0xE6, 0xAC, 0x88];
pub const VERSION: u8 = 1;
const HMAC_KEY: &[u8] = b"ji-dotfiles-hmac-key-v1";

pub const HEADER_SIZE: usize = 11;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CipherType {
    Age = 0,
    Pgp = 1,
}

impl CipherType {
    pub fn from_u8(v: u8) -> Result<Self> {
        match v {
            0 => Ok(CipherType::Age),
            1 => Ok(CipherType::Pgp),
            _ => Err(Error::Format(format!("unknown cipher type: {v}"))),
        }
    }

    pub fn from_config_type(s: &str) -> Result<Self> {
        match s {
            "age" => Ok(CipherType::Age),
            "pgp" => Ok(CipherType::Pgp),
            _ => Err(Error::Config(format!("unknown encryption type: {s}"))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IndexEntry {
    pub name: String,
    pub size: u64,
}

#[derive(Debug, Clone)]
pub struct PlainIndex {
    pub entries: Vec<IndexEntry>,
    pub total_size: u64,
}

pub fn write_header<W: Write>(writer: &mut W, cipher: CipherType, index_len: u32) -> Result<()> {
    writer
        .write_all(&MAGIC)
        .map_err(|e| Error::Format(format!("write magic: {e}")))?;
    writer
        .write_all(&[VERSION])
        .map_err(|e| Error::Format(format!("write version: {e}")))?;
    writer
        .write_all(&[cipher as u8])
        .map_err(|e| Error::Format(format!("write cipher: {e}")))?;
    writer
        .write_all(&[0u8; 2])
        .map_err(|e| Error::Format(format!("write reserved: {e}")))?;
    writer
        .write_all(&index_len.to_le_bytes())
        .map_err(|e| Error::Format(format!("write index_len: {e}")))?;
    Ok(())
}

pub fn read_header<R: Read>(reader: &mut R) -> Result<(CipherType, u32)> {
    let mut magic = [0u8; 3];
    reader
        .read_exact(&mut magic)
        .map_err(|e| Error::Format(format!("read magic: {e}")))?;
    if magic != MAGIC {
        return Err(Error::InvalidMagic);
    }

    let mut version = [0u8; 1];
    reader
        .read_exact(&mut version)
        .map_err(|e| Error::Format(format!("read version: {e}")))?;
    if version[0] != VERSION {
        return Err(Error::UnsupportedVersion(version[0]));
    }

    let mut cipher = [0u8; 1];
    reader
        .read_exact(&mut cipher)
        .map_err(|e| Error::Format(format!("read cipher: {e}")))?;
    let cipher = CipherType::from_u8(cipher[0])?;

    let mut reserved = [0u8; 2];
    reader
        .read_exact(&mut reserved)
        .map_err(|e| Error::Format(format!("read reserved: {e}")))?;
    if reserved != [0u8; 2] {
        return Err(Error::Format(
            "reserved bytes must be zero for version 1".into(),
        ));
    }

    let mut index_len_bytes = [0u8; 4];
    reader
        .read_exact(&mut index_len_bytes)
        .map_err(|e| Error::Format(format!("read index_len: {e}")))?;
    let index_len = u32::from_le_bytes(index_len_bytes);

    Ok((cipher, index_len))
}

pub fn write_index<W: Write>(writer: &mut W, index: &PlainIndex) -> Result<()> {
    let mut buf = Vec::new();

    let count = index.entries.len() as u32;
    buf.extend_from_slice(&count.to_le_bytes());

    for entry in &index.entries {
        let name_bytes = entry.name.as_bytes();
        let name_len = name_bytes.len() as u16;
        buf.extend_from_slice(&name_len.to_le_bytes());
        buf.extend_from_slice(name_bytes);
        buf.extend_from_slice(&entry.size.to_le_bytes());
    }

    buf.extend_from_slice(&index.total_size.to_le_bytes());

    let mut mac = Hmac::<Sha256>::new_from_slice(HMAC_KEY)
        .map_err(|e| Error::Format(format!("hmac init: {e}")))?;
    mac.update(&buf);
    let hmac_result = mac.finalize();
    buf.extend_from_slice(&hmac_result.into_bytes());

    writer
        .write_all(&buf)
        .map_err(|e| Error::Format(format!("write index: {e}")))?;

    Ok(())
}

pub fn read_index<R: Read>(reader: &mut R) -> Result<PlainIndex> {
    let mut buf = Vec::new();
    reader
        .read_to_end(&mut buf)
        .map_err(|e| Error::Format(format!("read index: {e}")))?;

    if buf.len() < 44 {
        return Err(Error::Format("index too short".into()));
    }

    const MAX_ENTRIES: u32 = 100_000;
    let count = u32::from_le_bytes(buf[..4].try_into().unwrap());
    if count > MAX_ENTRIES {
        return Err(Error::Format(format!(
            "index entry count too large: {count}"
        )));
    }

    let hmac_start = buf.len() - 32;
    let index_data = &buf[..hmac_start];
    let stored_hmac = &buf[hmac_start..];

    let mut mac = Hmac::<Sha256>::new_from_slice(HMAC_KEY)
        .map_err(|e| Error::Format(format!("hmac init: {e}")))?;
    mac.update(index_data);
    mac.verify_slice(stored_hmac)
        .map_err(|_| Error::HmacMismatch)?;

    let mut cursor = Cursor::new(index_data);

    let mut count_bytes = [0u8; 4];
    cursor
        .read_exact(&mut count_bytes)
        .map_err(|e| Error::Format(format!("read count: {e}")))?;
    let count = u32::from_le_bytes(count_bytes) as usize;

    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        let mut name_len_bytes = [0u8; 2];
        cursor
            .read_exact(&mut name_len_bytes)
            .map_err(|e| Error::Format(format!("read name len: {e}")))?;
        let name_len = u16::from_le_bytes(name_len_bytes) as usize;

        const MAX_NAME_LEN: usize = 4096;
        if name_len > MAX_NAME_LEN {
            return Err(Error::Format(format!("entry name too long: {name_len}")));
        }

        let mut name_bytes = vec![0u8; name_len];
        cursor
            .read_exact(&mut name_bytes)
            .map_err(|e| Error::Format(format!("read name: {e}")))?;
        let name = String::from_utf8(name_bytes)
            .map_err(|e| Error::Format(format!("invalid utf8 in name: {e}")))?;

        let mut size_bytes = [0u8; 8];
        cursor
            .read_exact(&mut size_bytes)
            .map_err(|e| Error::Format(format!("read size: {e}")))?;
        let size = u64::from_le_bytes(size_bytes);

        entries.push(IndexEntry { name, size });
    }

    let mut total_bytes = [0u8; 8];
    cursor
        .read_exact(&mut total_bytes)
        .map_err(|e| Error::Format(format!("read total size: {e}")))?;
    let total_size = u64::from_le_bytes(total_bytes);

    Ok(PlainIndex {
        entries,
        total_size,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_roundtrip() {
        let mut buf = Vec::new();
        write_header(&mut buf, CipherType::Age, 42).unwrap();
        assert_eq!(buf.len(), HEADER_SIZE);

        let mut cursor = Cursor::new(&buf);
        let (cipher, index_len) = read_header(&mut cursor).unwrap();
        assert_eq!(cipher, CipherType::Age);
        assert_eq!(index_len, 42);
    }

    #[test]
    fn invalid_magic_rejected() {
        let buf = vec![0u8; HEADER_SIZE];
        let err = read_header(&mut Cursor::new(&buf)).unwrap_err();
        assert!(matches!(err, Error::InvalidMagic));
    }

    #[test]
    fn index_roundtrip() {
        let index = PlainIndex {
            entries: vec![
                IndexEntry {
                    name: ".zshrc".into(),
                    size: 1234,
                },
                IndexEntry {
                    name: ".config/nvim/init.lua".into(),
                    size: 5678,
                },
            ],
            total_size: 6912,
        };

        let mut buf = Vec::new();
        write_index(&mut buf, &index).unwrap();

        let mut cursor = Cursor::new(&buf);
        let parsed = read_index(&mut cursor).unwrap();
        assert_eq!(parsed.entries.len(), 2);
        assert_eq!(parsed.entries[0].name, ".zshrc");
        assert_eq!(parsed.entries[0].size, 1234);
        assert_eq!(parsed.total_size, 6912);
    }

    #[test]
    fn index_hmac_tamper_detected() {
        let index = PlainIndex {
            entries: vec![IndexEntry {
                name: ".zshrc".into(),
                size: 100,
            }],
            total_size: 100,
        };

        let mut buf = Vec::new();
        write_index(&mut buf, &index).unwrap();

        buf[5] ^= 0xFF;

        let mut cursor = Cursor::new(&buf);
        let err = read_index(&mut cursor).unwrap_err();
        assert!(matches!(err, Error::HmacMismatch));
    }
}
