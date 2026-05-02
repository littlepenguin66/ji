use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Config {
    pub encryption: EncryptionConfig,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub remote: Vec<RemoteConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct EncryptionConfig {
    #[serde(rename = "type")]
    pub encryption_type: String,
    pub recipients: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct RemoteConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub remote_type: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

impl RemoteConfig {
    pub fn build(&self) -> Result<Box<dyn crate::remote::Remote>> {
        match self.remote_type.as_str() {
            "webdav" => Ok(Box::new(crate::remote::webdav::WebdavRemote {
                url: self.url.clone(),
                user: self.user.clone(),
            })),
            "ssh" => {
                let (host, path) = crate::remote::parse_ssh_url(&self.url)?;
                Ok(Box::new(crate::remote::ssh::SshRemote {
                    host,
                    user: self.user.clone(),
                    path,
                }))
            }
            _ => Err(crate::error::Error::Remote(format!(
                "unsupported remote type: {}",
                self.remote_type
            ))),
        }
    }
}

impl Config {
    pub fn new(recipients: Vec<String>) -> Self {
        Config {
            encryption: EncryptionConfig {
                encryption_type: "age".to_string(),
                recipients,
            },
            remote: Vec::new(),
        }
    }

    pub fn read(path: &Path) -> Result<Self> {
        let content =
            std::fs::read_to_string(path).map_err(|e| Error::Config(format!("read: {e}")))?;
        toml::from_str(&content).map_err(|e| Error::Config(format!("parse: {e}")))
    }

    pub fn write(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| Error::Config(format!("mkdir: {e}")))?;
        }
        let content = toml::to_string_pretty(self)?;
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, &content).map_err(|e| Error::Config(format!("write tmp: {e}")))?;
        std::fs::rename(&tmp, path).map_err(|e| Error::Config(format!("rename: {e}")))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_roundtrip() {
        let cfg = Config::new(vec!["age1abc123".into()]);
        let serialized = toml::to_string_pretty(&cfg).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();
        assert_eq!(cfg, deserialized);
        assert_eq!(deserialized.encryption.encryption_type, "age");
        assert_eq!(deserialized.encryption.recipients, vec!["age1abc123"]);
    }

    #[test]
    fn config_with_remote() {
        let mut cfg = Config::new(vec!["age1abc123".into()]);
        cfg.remote.push(RemoteConfig {
            name: "nas".into(),
            remote_type: "webdav".into(),
            url: "https://nas.local/ji/".into(),
            user: Some("jrz".into()),
        });
        let serialized = toml::to_string_pretty(&cfg).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();
        assert_eq!(cfg, deserialized);
        assert_eq!(deserialized.remote.len(), 1);
        assert_eq!(deserialized.remote[0].name, "nas");
    }

    #[test]
    fn read_write_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let cfg = Config::new(vec!["age1xyz".into()]);
        cfg.write(&path).unwrap();
        assert!(path.exists());

        let loaded = Config::read(&path).unwrap();
        assert_eq!(cfg, loaded);
    }
}
