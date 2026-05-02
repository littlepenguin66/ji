use crate::error::Result;
use crate::store::manifest::Manifest;
use crate::store::path;

pub fn run(json: bool) -> Result<()> {
    let manifest = Manifest::read(&path::manifest_toml())?;

    if json {
        let output = serde_json::to_string_pretty(&manifest.files)?;
        println!("{output}");
    } else {
        if manifest.files.is_empty() {
            println!("(no files tracked)");
            return Ok(());
        }
        for path in manifest.list_paths() {
            if let Some(entry) = manifest.get(path) {
                println!("{}  {}", entry.checksum, path);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_json_output() {
        let _guard = crate::store::path::TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        crate::store::path::with_test_home(tmp.path(), || {
            let mut m = Manifest::new();
            m.add(".zshrc", "abc123".into());
            m.write(&path::manifest_toml()).unwrap();
            run(true).expect("list --json");
        });
    }

    #[test]
    fn list_empty() {
        let _guard = crate::store::path::TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        crate::store::path::with_test_home(tmp.path(), || {
            run(false).expect("list");
        });
    }
}
