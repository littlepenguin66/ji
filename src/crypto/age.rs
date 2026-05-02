use crate::crypto::Cipher;
use crate::error::{Error, Result};
use age::secrecy::ExposeSecret;
use age::x25519::Identity;
use std::io::{Cursor, Read, Write};

pub struct AgeCipher;

impl AgeCipher {
    pub fn generate_identity() -> (String, String) {
        let identity = Identity::generate();
        let pubkey = identity.to_public();
        let priv_str = identity.to_string().expose_secret().to_string();
        (priv_str, pubkey.to_string())
    }
}

impl Cipher for AgeCipher {
    fn encrypt(data: &[u8], recipients: &[String]) -> Result<Vec<u8>> {
        if recipients.is_empty() {
            return Err(Error::Crypto("no recipients provided".into()));
        }

        let recs = parse_recipients(recipients)?;

        let encryptor = age::Encryptor::with_recipients(recs.iter().map(|r| r.as_ref()))
            .map_err(|e| Error::Crypto(format!("encrypt setup: {e}")))?;

        let mut output = vec![];
        let mut writer = encryptor
            .wrap_output(&mut output)
            .map_err(|e| Error::Crypto(format!("wrap output: {e}")))?;
        writer
            .write_all(data)
            .map_err(|e| Error::Crypto(format!("write: {e}")))?;
        writer
            .finish()
            .map_err(|e| Error::Crypto(format!("finish: {e}")))?;

        Ok(output)
    }

    fn decrypt(data: &[u8]) -> Result<Vec<u8>> {
        let identities = load_identities()?;

        let reader = Cursor::new(data);
        let decryptor = age::Decryptor::new(reader)
            .map_err(|e| Error::Crypto(format!("decrypt setup: {e}")))?;

        let mut output = vec![];
        let mut decrypt_reader = decryptor
            .decrypt(identities.iter().map(|id| id.as_ref()))
            .map_err(|e| Error::Crypto(format!("decrypt: {e}")))?;
        decrypt_reader
            .read_to_end(&mut output)
            .map_err(|e| Error::Crypto(format!("read: {e}")))?;

        Ok(output)
    }

    fn list_recipients(data: &[u8]) -> Result<Vec<String>> {
        let text = String::from_utf8_lossy(data);
        let recipients: Vec<String> = text
            .lines()
            .skip(1)
            .take_while(|line| !line.starts_with("---"))
            .filter(|line| {
                line.starts_with("-> X25519 ")
                    || line.starts_with("-> ssh-rsa ")
                    || line.starts_with("-> ssh-ed25519 ")
            })
            .map(|line| line[3..].trim().to_string())
            .collect();
        Ok(recipients)
    }
}

fn parse_recipients(keys: &[String]) -> Result<Vec<Box<dyn age::Recipient>>> {
    let mut recs: Vec<Box<dyn age::Recipient>> = Vec::new();
    for r in keys {
        if let Ok(native) = r.parse::<age::x25519::Recipient>() {
            recs.push(Box::new(native));
            continue;
        }
        match r.parse::<age::ssh::Recipient>() {
            Ok(ssh) => recs.push(Box::new(ssh)),
            Err(e) => return Err(Error::Crypto(format!("parse recipient: {e:?}"))),
        }
    }
    Ok(recs)
}

fn load_identities() -> Result<Vec<Box<dyn age::Identity>>> {
    let mut identities: Vec<Box<dyn age::Identity>> = Vec::new();

    let identity_path = crate::store::path::identity_path();
    if identity_path.exists() {
        let content = std::fs::read_to_string(&identity_path)
            .map_err(|e| Error::Crypto(format!("read identity: {e}")))?;
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Ok(id) = line.parse::<Identity>() {
                identities.push(Box::new(id));
            }
        }
    }

    if let Ok(ssh_ids) = load_ssh_identities() {
        identities.extend(ssh_ids);
    }

    if identities.is_empty() {
        return Err(Error::NoPrivateKey);
    }

    Ok(identities)
}

fn load_ssh_identities() -> Result<Vec<Box<dyn age::Identity>>> {
    let mut ids = Vec::new();
    let ssh_dir = crate::store::path::home_dir().join(".ssh");

    if !ssh_dir.exists() {
        return Ok(ids);
    }

    for entry in
        std::fs::read_dir(&ssh_dir).map_err(|e| Error::Crypto(format!("read ssh dir: {e}")))?
    {
        let entry = entry.map_err(|e| Error::Crypto(format!("ssh entry: {e}")))?;
        let path = entry.path();

        if !path.is_file()
            || path.extension().is_none_or(|e| e == "pub")
            || path
                .file_name()
                .is_none_or(|n| n == "known_hosts" || n == "authorized_keys")
        {
            continue;
        }

        let key_data = std::fs::read(&path).map_err(|e| Error::Crypto(format!("read ssh: {e}")))?;
        match age::ssh::Identity::from_buffer(Cursor::new(&key_data), None) {
            Ok(ssh_id) => {
                ids.push(Box::new(ssh_id));
            }
            Err(_) => continue,
        }
    }

    Ok(ids)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_identity_produces_valid_keys() {
        let (priv_key, pub_key) = AgeCipher::generate_identity();
        assert!(priv_key.starts_with("AGE-SECRET-KEY-1"));
        assert!(pub_key.starts_with("age1"));
    }

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let (priv_key, pub_key) = AgeCipher::generate_identity();

        let data = b"hello world this is test data";
        let recipients = vec![pub_key];

        let encrypted = AgeCipher::encrypt(data, &recipients).expect("encrypt should succeed");
        assert!(!encrypted.is_empty());

        let identity: Identity = priv_key.parse().expect("parse identity");
        let decryptor = age::Decryptor::new(Cursor::new(&encrypted[..])).expect("decryptor");
        let mut output = vec![];
        let mut reader = decryptor
            .decrypt(std::iter::once(&identity as &dyn age::Identity))
            .expect("decrypt");
        reader.read_to_end(&mut output).expect("read");

        assert_eq!(output, data);
    }

    #[test]
    fn list_recipients_from_encrypted() {
        let (_priv_key, pub_key) = AgeCipher::generate_identity();
        let data = b"test data";
        let encrypted = AgeCipher::encrypt(data, &[pub_key]).expect("encrypt");

        let recipients = AgeCipher::list_recipients(&encrypted).expect("list recipients");
        assert_eq!(recipients.len(), 1);
        assert!(recipients[0].starts_with("X25519 "));
    }

    #[test]
    fn encrypt_to_multiple_recipients() {
        let (_priv1, pub1) = AgeCipher::generate_identity();
        let (_priv2, pub2) = AgeCipher::generate_identity();

        let data = b"multi-recipient test";
        let encrypted = AgeCipher::encrypt(data, &[pub1, pub2]).expect("encrypt");

        let recipients = AgeCipher::list_recipients(&encrypted).expect("list recipients");
        assert_eq!(recipients.len(), 2);
    }
}
