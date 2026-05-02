
#![allow(dead_code)]

use crate::crypto::Cipher;
use crate::error::{Error, Result};

pub struct PgpCipher;

impl Cipher for PgpCipher {
    #[cfg(feature = "pgp")]
    fn encrypt(data: &[u8], recipients: &[String]) -> Result<Vec<u8>> {
        use sequoia_openpgp::cert::prelude::*;
        use sequoia_openpgp::serialize::stream::{Armorer, Encryptor, LiteralWriter, Message};
        use std::io::Write;

        if recipients.is_empty() {
            return Err(Error::Crypto("no recipients provided".into()));
        }

        let mut recipient_keys = Vec::new();
        for r in recipients {
            let cert = sequoia_openpgp::Cert::from_bytes(r.as_bytes())
                .map_err(|e| Error::Crypto(format!("parse pgp cert: {e}")))?;
            for key in cert.keys().for_transport_encryption() {
                recipient_keys.push(key.key().clone());
            }
        }

        if recipient_keys.is_empty() {
            return Err(Error::Crypto("no valid PGP encryption keys found".into()));
        }

        let mut sink = Vec::new();

        let message = Message::new(&mut sink);
        let armorer = Armorer::new(message)
            .build()
            .map_err(|e| Error::Crypto(format!("pgp armor: {e}")))?;

        let encryptor = Encryptor::for_recipients(armorer, recipient_keys.iter())
            .build()
            .map_err(|e| Error::Crypto(format!("pgp encrypt: {e}")))?;

        let mut writer = LiteralWriter::new(encryptor)
            .build()
            .map_err(|e| Error::Crypto(format!("pgp literal: {e}")))?;

        writer
            .write_all(data)
            .map_err(|e| Error::Crypto(format!("pgp write: {e}")))?;
        writer
            .finalize()
            .map_err(|e| Error::Crypto(format!("pgp finalize: {e}")))?;

        Ok(sink)
    }

    #[cfg(not(feature = "pgp"))]
    fn encrypt(_data: &[u8], _recipients: &[String]) -> Result<Vec<u8>> {
        Err(Error::Crypto(
            "PGP support not compiled. Rebuild with --features pgp".into(),
        ))
    }

    #[cfg(feature = "pgp")]
    fn decrypt(data: &[u8]) -> Result<Vec<u8>> {
        use sequoia_openpgp::parse::stream::DecryptorBuilder;
        use sequoia_openpgp::policy::StandardPolicy;
        use std::io::Read;

        let certs = load_pgp_secret_keys()?;
        let policy = StandardPolicy::new();

        for cert in &certs {
            if let Ok(mut decryptor) =
                DecryptorBuilder::from_bytes(data)?.with_policy(&policy, None, [cert.clone()])
            {
                let mut buf = Vec::new();
                if decryptor.read_to_end(&mut buf).is_ok() {
                    return Ok(buf);
                }
            }
        }

        Err(Error::NoPrivateKey)
    }

    #[cfg(not(feature = "pgp"))]
    fn decrypt(_data: &[u8]) -> Result<Vec<u8>> {
        Err(Error::Crypto(
            "PGP support not compiled. Rebuild with --features pgp".into(),
        ))
    }

    #[cfg(feature = "pgp")]
    fn list_recipients(data: &[u8]) -> Result<Vec<String>> {
        use sequoia_openpgp::parse::stream::DecryptorBuilder;

        let reader = DecryptorBuilder::from_bytes(data)
            .map_err(|e| Error::Crypto(format!("pgp parse: {e}")))?;

        let recipients: Vec<String> = reader
            .get_recipients()
            .iter()
            .map(|r| format!("{:X}", r))
            .collect();

        Ok(recipients)
    }

    #[cfg(not(feature = "pgp"))]
    fn list_recipients(_data: &[u8]) -> Result<Vec<String>> {
        Err(Error::Crypto(
            "PGP support not compiled. Rebuild with --features pgp".into(),
        ))
    }
}

#[cfg(feature = "pgp")]
fn load_pgp_secret_keys() -> Result<Vec<sequoia_openpgp::Cert>> {
    let mut certs = Vec::new();

    if let Some(home) = dirs::home_dir() {
        let gnupg = home.join(".gnupg");

        for file in &["pubring.gpg", "pubring.kbx", "secring.gpg"] {
            let path = gnupg.join(file);
            if let Ok(data) = std::fs::read(&path) {
                if let Ok(iter) = sequoia_openpgp::CertParser::from_bytes(&data) {
                    for r in iter {
                        if let Ok(cert) = r {
                            if cert.keys().secret().next().is_some() {
                                certs.push(cert);
                            }
                        }
                    }
                }
            }
        }

        let private_dir = gnupg.join("private-keys-v1.d");
        if let Ok(entries) = std::fs::read_dir(&private_dir) {
            for entry in entries.flatten() {
                if let Ok(data) = std::fs::read(entry.path()) {
                    if let Ok(cert) = sequoia_openpgp::Cert::from_bytes(&data) {
                        if cert.keys().secret().next().is_some() {
                            certs.push(cert);
                        }
                    }
                }
            }
        }
    }

    if certs.is_empty() {
        return Err(Error::NoPrivateKey);
    }

    Ok(certs)
}
