use crate::error::Result;
use crate::store::manifest::Manifest;
use crate::store::path;
use std::path::PathBuf;

pub fn run(paths: Vec<PathBuf>, all: bool) -> Result<()> {
    let manifest_path = path::manifest_toml();
    let mut manifest = Manifest::read(&manifest_path)?;

    if all {
        manifest.remove_all();
        manifest.write(&manifest_path)?;
        println!("Removed all tracked files.");
        return Ok(());
    }

    for raw_path in &paths {
        let rel = raw_path.to_string_lossy().to_string();
        match manifest.remove(&rel) {
            Ok(()) => println!("removed: {rel}"),
            Err(e) => eprintln!("ji: {e}"),
        }
    }

    manifest.write(&manifest_path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn rm_tracked_file() {
        let _guard = crate::store::path::TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        crate::store::path::with_test_home(tmp.path(), || {

        // Seed a manifest
        let mut m = Manifest::new();
        m.add(".zshrc", "abc".into());
        m.add(".gitconfig", "def".into());
        m.write(&path::manifest_toml()).unwrap();

        run(vec![PathBuf::from(".zshrc")], false).expect("rm");

        let manifest = Manifest::read(&path::manifest_toml()).unwrap();
        assert!(!manifest.is_tracked(".zshrc"));
        assert!(manifest.is_tracked(".gitconfig"));

        });
    }

    #[test]
    fn rm_all() {
        let _guard = crate::store::path::TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        crate::store::path::with_test_home(tmp.path(), || {

        let mut m = Manifest::new();
        m.add(".zshrc", "abc".into());
        m.add(".gitconfig", "def".into());
        m.write(&path::manifest_toml()).unwrap();

        run(vec![], true).expect("rm --all");

        let manifest = Manifest::read(&path::manifest_toml()).unwrap();
        assert!(manifest.files.is_empty());

        });
    }
}
