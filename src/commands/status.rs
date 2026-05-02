use crate::error::Result;
use crate::store::manifest::{self, Manifest};
use crate::store::path;

pub fn run(short: bool) -> Result<()> {
    let manifest = Manifest::read(&path::manifest_toml())?;

    if manifest.files.is_empty() {
        if !short {
            println!("(no files tracked)");
        }
        return Ok(());
    }

    let statuses = manifest::compute_status(&manifest)?;

    let mut has_changes = false;
    for entry in &statuses {
        if entry.status == manifest::FileStatus::Unchanged {
            continue;
        }
        has_changes = true;
        println!("{} {}", entry.status, entry.path);
    }

    if !has_changes && !short {
        println!("(no changes)");
    }

    if has_changes {
        return Err(crate::error::Error::HasChanges);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_empty_manifest() {
        let _guard = crate::store::path::TEST_MUTEX
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        crate::store::path::with_test_home(tmp.path(), || {
            let m = Manifest::new();
            m.write(&path::manifest_toml()).unwrap();

            run(false).expect("status");
        });
    }

    #[test]
    fn status_short() {
        let _guard = crate::store::path::TEST_MUTEX
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        crate::store::path::with_test_home(tmp.path(), || {
            let file_path = tmp.path().join(".zshrc");
            std::fs::write(&file_path, "export EDITOR=nvim").unwrap();

            let mut m = Manifest::new();
            let checksum = crate::store::manifest::compute_checksum(&file_path).unwrap();
            m.add(".zshrc", checksum);
            m.write(&path::manifest_toml()).unwrap();

            run(true).expect("status --short");
        });
    }
}
