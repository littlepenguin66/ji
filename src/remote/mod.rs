pub mod ssh;
pub mod webdav;

use crate::error::Result;
use std::path::Path;

pub struct RemoteEntry {
    pub name: String,
    pub size: u64,
    pub modified: chrono::DateTime<chrono::Utc>,
}

pub trait Remote {
    fn push(&self, local: &Path, name: &str) -> Result<()>;
    fn pull(&self, name: &str, local: &Path) -> Result<()>;
    fn list(&self) -> Result<Vec<RemoteEntry>>;
    fn delete(&self, name: &str) -> Result<()>;
    fn test(&self) -> Result<()>;
}

/// Parse an SSH url of the form `host:/path` or `host:`.
pub fn parse_ssh_url(url: &str) -> Result<(String, String)> {
    let (host, path) = url
        .split_once(':')
        .ok_or_else(|| {
            crate::error::Error::Remote(format!(
                "invalid ssh url: {url} (expected host:/path)"
            ))
        })?;
    Ok((host.to_string(), path.to_string()))
}
