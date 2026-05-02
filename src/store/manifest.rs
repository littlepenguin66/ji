use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Manifest {
    pub files: HashMap<String, FileEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct FileEntry {
    pub checksum: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct StatusEntry {
    pub path: String,
    pub status: FileStatus,
    pub stored_checksum: Option<String>,
    pub current_checksum: Option<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
#[allow(dead_code)]
pub enum FileStatus {
    Unchanged,
    Modified,
    Deleted,
    Added, // in manifest but missing on disk
}

impl std::fmt::Display for FileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileStatus::Unchanged => write!(f, " "),
            FileStatus::Modified => write!(f, "M"),
            FileStatus::Deleted => write!(f, "-"),
            FileStatus::Added => write!(f, "A"),
        }
    }
}

impl Manifest {
    pub fn new() -> Self {
        Manifest {
            files: HashMap::new(),
        }
    }

    pub fn read(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Manifest::new());
        }
        let content =
            std::fs::read_to_string(path).map_err(|e| Error::Manifest(format!("read: {e}")))?;
        toml::from_str(&content).map_err(|e| Error::Manifest(format!("parse: {e}")))
    }

    pub fn write(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| Error::Manifest(format!("mkdir: {e}")))?;
        }
        let content = toml::to_string_pretty(self)?;
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, &content).map_err(|e| Error::Manifest(format!("write tmp: {e}")))?;
        std::fs::rename(&tmp, path).map_err(|e| Error::Manifest(format!("rename: {e}")))?;
        Ok(())
    }

    pub fn add(&mut self, path: &str, checksum: String) {
        self.files.insert(
            path.to_string(),
            FileEntry { checksum },
        );
    }

    pub fn remove(&mut self, path: &str) -> Result<()> {
        if self.files.remove(path).is_none() {
            return Err(Error::NotTracked(PathBuf::from(path)));
        }
        Ok(())
    }

    pub fn remove_all(&mut self) {
        self.files.clear();
    }

    pub fn list_paths(&self) -> Vec<&String> {
        let mut paths: Vec<&String> = self.files.keys().collect();
        paths.sort();
        paths
    }

    pub fn get(&self, path: &str) -> Option<&FileEntry> {
        self.files.get(path)
    }

    pub fn is_tracked(&self, path: &str) -> bool {
        self.files.contains_key(path)
    }
}

/// Compute SHA-256 checksum of a file.
pub fn compute_checksum(path: &Path) -> Result<String> {
    let mut file =
        std::fs::File::open(path).map_err(|e| Error::Manifest(format!("open {path:?}: {e}")))?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = file
            .read(&mut buf)
            .map_err(|e| Error::Manifest(format!("read {path:?}: {e}")))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

/// Compute SHA-256 checksum from in-memory bytes.
pub fn compute_checksum_reader(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Returns the effective HOME directory (respects JI_TEST_HOME for testing).
fn home_dir() -> PathBuf {
    if let Ok(test_home) = std::env::var("JI_TEST_HOME") {
        PathBuf::from(test_home)
    } else {
        dirs::home_dir().expect("could not determine HOME")
    }
}

/// Resolve a path relative to $HOME. Returns the absolute path.
pub fn resolve_home(relative: &str) -> PathBuf {
    home_dir().join(relative.trim_start_matches('/'))
}

/// Get the relative path from $HOME.
pub fn relativize(abs: &Path) -> Result<String> {
    let home = home_dir();
    abs.strip_prefix(&home)
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|_| Error::Manifest(format!("path {abs:?} is not under HOME")))
}

/// Compute status for all tracked files.
pub fn compute_status(manifest: &Manifest) -> Result<Vec<StatusEntry>> {
    let mut entries = Vec::new();

    for (path, entry) in &manifest.files {
        let abs = resolve_home(path);
        if !abs.exists() {
            entries.push(StatusEntry {
                path: path.clone(),
                status: FileStatus::Deleted,
                stored_checksum: Some(entry.checksum.clone()),
                current_checksum: None,
            });
        } else {
            let current = compute_checksum(&abs)?;
            let status = if current == entry.checksum {
                FileStatus::Unchanged
            } else {
                FileStatus::Modified
            };
            entries.push(StatusEntry {
                path: path.clone(),
                status,
                stored_checksum: Some(entry.checksum.clone()),
                current_checksum: Some(current),
            });
        }
    }

    entries.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_add_and_list() {
        let mut m = Manifest::new();
        m.add(".zshrc", "abc123".into());
        m.add(".gitconfig", "def456".into());

        assert_eq!(m.list_paths(), vec![".gitconfig", ".zshrc"]);
        assert!(m.is_tracked(".zshrc"));
        assert!(!m.is_tracked(".bashrc"));
    }

    #[test]
    fn manifest_remove() {
        let mut m = Manifest::new();
        m.add(".zshrc", "abc123".into());
        m.remove(".zshrc").expect("remove tracked");
        assert!(m.files.is_empty());
    }

    #[test]
    fn manifest_remove_not_tracked() {
        let mut m = Manifest::new();
        let err = m.remove(".nonexistent").unwrap_err();
        assert!(matches!(err, Error::NotTracked(_)));
    }

    #[test]
    fn manifest_remove_all() {
        let mut m = Manifest::new();
        m.add(".zshrc", "abc".into());
        m.add(".gitconfig", "def".into());
        m.remove_all();
        assert!(m.files.is_empty());
    }

    #[test]
    fn manifest_roundtrip() {
        let mut m = Manifest::new();
        m.add(".zshrc", "sha256:abc123".into());
        m.add(".config/nvim/init.lua", "sha256:def456".into());

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("manifest.toml");
        m.write(&path).unwrap();

        let loaded = Manifest::read(&path).unwrap();
        assert_eq!(m.files, loaded.files);
    }

    #[test]
    fn manifest_read_nonexistent_returns_empty() {
        let m = Manifest::read(Path::new("/nonexistent/manifest.toml")).unwrap();
        assert!(m.files.is_empty());
    }

    #[test]
    fn compute_checksum_known_content() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");
        std::fs::write(&path, "hello world").unwrap();

        let cs = compute_checksum(&path).unwrap();
        // SHA-256 of "hello world" is known
        assert_eq!(
            cs,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn compute_status_detects_changes() {
        let dir = tempfile::tempdir().unwrap();
        let test_file = dir.path().join("test.txt");

        // Create a file and compute its checksum
        std::fs::write(&test_file, "version 1").unwrap();

        // We can only test status with real HOME for now.
        // This is tested more in integration tests.
    }
}
