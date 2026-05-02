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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ssh_url_with_path() {
        let (host, path) = parse_ssh_url("vps.example.com:/home/jrz/ji").unwrap();
        assert_eq!(host, "vps.example.com");
        assert_eq!(path, "/home/jrz/ji");
    }

    #[test]
    fn parse_ssh_url_root_path() {
        let (host, path) = parse_ssh_url("localhost:/").unwrap();
        assert_eq!(host, "localhost");
        assert_eq!(path, "/");
    }

    #[test]
    fn parse_ssh_url_no_path() {
        let (host, path) = parse_ssh_url("myserver:").unwrap();
        assert_eq!(host, "myserver");
        assert_eq!(path, "");
    }

    #[test]
    fn parse_ssh_url_no_colon_is_error() {
        let err = parse_ssh_url("not-a-valid-url").unwrap_err();
        assert!(err.to_string().contains("host:/path"));
    }

    #[test]
    fn parse_ssh_url_multiple_colons() {
        let (host, path) = parse_ssh_url("host:/path:with:colons").unwrap();
        assert_eq!(host, "host");
        assert_eq!(path, "/path:with:colons");
    }
}
