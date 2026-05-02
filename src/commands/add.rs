use crate::error::Result;
use crate::store::manifest::{self, Manifest};
use crate::store::path;
use std::path::PathBuf;

pub fn run(paths: Vec<PathBuf>, include: Vec<String>, exclude: Vec<String>) -> Result<()> {
    let manifest_path = path::manifest_toml();
    let mut manifest = Manifest::read(&manifest_path)?;

    for raw_path in &paths {
        let rel = if raw_path.is_absolute() {
            manifest::relativize(raw_path)?
        } else {
            raw_path.to_string_lossy().to_string()
        };

        if manifest.is_tracked(&rel) {
            return Err(crate::error::Error::AlreadyTracked(PathBuf::from(&rel)));
        }

        let abs = manifest::resolve_home(&rel);

        if abs.is_dir() {
            add_directory(&mut manifest, &abs, &rel, &include, &exclude)?;
        } else if abs.is_file() {
            add_file(&mut manifest, &abs, &rel, &include, &exclude)?;
        } else {
            eprintln!("ji: warning: '{}' not found, skipping", rel);
        }
    }

    manifest.write(&manifest_path)?;
    Ok(())
}

fn add_file(
    manifest: &mut Manifest,
    abs: &PathBuf,
    rel: &str,
    include: &[String],
    exclude: &[String],
) -> Result<()> {
    if crate::store::ignore::is_ignored(rel) {
        return Ok(());
    }
    if !should_include(rel, include, exclude) {
        return Ok(());
    }

    let checksum = manifest::compute_checksum(abs)?;
    manifest.add(rel, checksum);
    Ok(())
}

fn add_directory(
    manifest: &mut Manifest,
    abs: &PathBuf,
    _base_rel: &str,
    include: &[String],
    exclude: &[String],
) -> Result<()> {
    let home = crate::store::path::home_dir();

    for entry in walkdir::WalkDir::new(abs)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_dir() {
            continue;
        }

        let entry_abs = entry.path().to_path_buf();
        let entry_rel = entry_abs
            .strip_prefix(&home)
            .unwrap_or(&entry_abs)
            .to_string_lossy()
            .to_string();

        add_file(manifest, &entry_abs.to_path_buf(), &entry_rel, include, exclude)?;
    }
    Ok(())
}

fn should_include(rel: &str, include: &[String], exclude: &[String]) -> bool {
    if !include.is_empty() {
        let matched = include.iter().any(|pat| {
            glob::Pattern::new(pat)
                .map(|p| p.matches(rel))
                .unwrap_or(false)
        });
        if !matched {
            return false;
        }
    }

    if !exclude.is_empty() {
        let matched = exclude.iter().any(|pat| {
            glob::Pattern::new(pat)
                .map(|p| p.matches(rel))
                .unwrap_or(false)
        });
        if matched {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn add_single_file() {
        let _guard = crate::store::path::TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        crate::store::path::with_test_home(tmp.path(), || {

        let file_path = tmp.path().join(".zshrc");
        std::fs::write(&file_path, "export EDITOR=nvim").unwrap();

        run(vec![PathBuf::from(".zshrc")], vec![], vec![]).expect("add");

        let manifest = Manifest::read(&path::manifest_toml()).unwrap();
        assert!(manifest.is_tracked(".zshrc"));

        });
    }

    #[test]
    fn add_nonexistent_warns() {
        let _guard = crate::store::path::TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        crate::store::path::with_test_home(tmp.path(), || {

        run(vec![PathBuf::from(".nonexistent")], vec![], vec![]).expect("add");

        let manifest = Manifest::read(&path::manifest_toml()).unwrap();
        assert!(manifest.files.is_empty());

        });
    }
}
