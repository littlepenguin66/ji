use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("config: {0}")]
    Config(String),

    #[error("manifest: {0}")]
    Manifest(String),

    #[error("crypto: {0}")]
    Crypto(String),

    #[error("archive: {0}")]
    Archive(String),

    #[error("format: {0}")]
    Format(String),

    #[error("remote: {0}")]
    Remote(String),

    #[error("'{0}' not tracked")]
    NotTracked(PathBuf),

    #[error("'{0}' already tracked")]
    AlreadyTracked(PathBuf),

    #[error("checksum mismatch for '{path}': expected {expected}, got {got}")]
    ChecksumMismatch {
        path: PathBuf,
        expected: String,
        got: String,
    },

    #[error("no private key available for decryption")]
    NoPrivateKey,

    #[error("invalid magic bytes: expected 笈")]
    InvalidMagic,

    #[error("unsupported version: {0}")]
    UnsupportedVersion(u8),

    #[error("HMAC verification failed")]
    HmacMismatch,

    #[error("toml serialize: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("toml deserialize: {0}")]
    TomlDeserialize(#[from] toml::de::Error),

    #[error("json: {0}")]
    Json(#[from] serde_json::Error),

    #[error("{0}")]
    Other(String),

    #[error("files have changed")]
    HasChanges,
}

pub type Result<T> = std::result::Result<T, Error>;
