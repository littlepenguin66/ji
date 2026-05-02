use crate::archive;
use crate::error::Result;
use crate::store::config::Config;
use crate::store::manifest::Manifest;
use crate::store::path;
use std::path::PathBuf;

pub fn run(output: Option<PathBuf>, strict: bool, verbose: bool) -> Result<()> {
    let manifest = Manifest::read(&path::manifest_toml())?;

    if manifest.files.is_empty() {
        eprintln!("ji: no files tracked");
        return Ok(());
    }

    // Get recipients from config
    let config = Config::read(&path::config_toml())?;
    if config.encryption.recipients.is_empty() {
        eprintln!("ji: no recipients configured. Run 'ji init' first.");
        return Ok(());
    }

    // Check checksums if strict or verbose
    if strict || verbose {
        let statuses = crate::store::manifest::compute_status(&manifest)?;
        let mismatches: Vec<_> = statuses
            .iter()
            .filter(|e| e.status != crate::store::manifest::FileStatus::Unchanged)
            .collect();

        if !mismatches.is_empty() {
            if strict {
                let first = &mismatches[0];
                return Err(crate::error::Error::ChecksumMismatch {
                    path: PathBuf::from(&first.path),
                    expected: first
                        .stored_checksum
                        .clone()
                        .unwrap_or_else(|| "-".into()),
                    got: first
                        .current_checksum
                        .clone()
                        .unwrap_or_else(|| "-".into()),
                });
            }
            if verbose {
                eprintln!("Changed files:");
                for e in &mismatches {
                    eprintln!("  {} {}", e.status, e.path);
                }
            }
        }
    }

    let output_path = output.unwrap_or_else(|| {
        let host = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".into());
        path::default_output(&host)
    });

    let cipher = crate::archive::format::CipherType::from_config_type(
        &config.encryption.encryption_type,
    )?;
    archive::pack_archive(
        &output_path,
        &manifest,
        &config.encryption.recipients,
        cipher,
    )?;

    println!("Packed to {}", output_path.display());

    // Update cache for diff (Step 7)
    update_cache(&manifest).ok();

    Ok(())
}

fn update_cache(manifest: &Manifest) -> Result<()> {
    let cache_dir = path::cache_dir();
    std::fs::create_dir_all(&cache_dir).map_err(|e| crate::error::Error::Io(e))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&cache_dir, std::fs::Permissions::from_mode(0o700)).ok();
    }

    for rel_path in manifest.list_paths() {
        let abs = crate::store::manifest::resolve_home(rel_path);
        if !abs.exists() {
            continue;
        }
        let cache_path = cache_dir.join(rel_path);
        if let Some(parent) = cache_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::copy(&abs, &cache_path).ok();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&cache_path, std::fs::Permissions::from_mode(0o600)).ok();
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set_test_home(path: &std::path::Path) {
        unsafe { std::env::set_var("JI_TEST_HOME", path.as_os_str()) };
    }

    fn unset_test_home() {
        unsafe { std::env::remove_var("JI_TEST_HOME") };
    }

    #[test]
    fn pack_empty_manifest_warns() {
        let _guard = crate::store::path::TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        set_test_home(tmp.path());

        // Create config with recipients
        let cfg = Config::new(vec!["age1test".into()]);
        cfg.write(&path::config_toml()).unwrap();

        run(None, false, false).expect("pack empty");

        unset_test_home();
    }

    #[test]
    fn pack_roundtrip() {
        let _guard = crate::store::path::TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        set_test_home(tmp.path());

        // Generate real age keypair
        let (priv_key, pub_key) = crate::crypto::age::AgeCipher::generate_identity();

        // Save identity for later decryption
        std::fs::create_dir_all(path::data_dir()).unwrap();
        std::fs::write(path::identity_path(), &priv_key).unwrap();

        let cfg = Config::new(vec![pub_key]);
        cfg.write(&path::config_toml()).unwrap();

        let file_path = tmp.path().join(".zshrc");
        std::fs::write(&file_path, "export EDITOR=nvim\n").unwrap();

        let checksum = crate::store::manifest::compute_checksum(&file_path).unwrap();
        let mut manifest = Manifest::new();
        manifest.add(".zshrc", checksum);
        manifest.write(&path::manifest_toml()).unwrap();

        let output = tmp.path().join("test.ji");
        run(Some(output.clone()), false, false).expect("pack");

        assert!(output.exists());
        assert!(output.metadata().unwrap().len() > 0);

        // Now unpack to verify roundtrip
        let restore_dir = tmp.path().join("restore");
        std::fs::create_dir_all(restore_dir.join(".local").join("share").join("ji")).unwrap();
        std::fs::copy(
            path::identity_path(),
            restore_dir.join(".local/share/ji/ji.identity.age"),
        ).unwrap();
        unsafe { std::env::set_var("JI_TEST_HOME", restore_dir.as_os_str()) };

        let restored = crate::archive::unpack_archive(&output, false, true, false, false)
            .expect("unpack");
        assert_eq!(restored, 1);

        let restored_file = restore_dir.join(".zshrc");
        assert!(restored_file.exists());
        let content = std::fs::read_to_string(&restored_file).unwrap();
        assert_eq!(content, "export EDITOR=nvim\n");

        // Reset test home
        unsafe { std::env::set_var("JI_TEST_HOME", tmp.path().as_os_str()) };
        unset_test_home();
    }
}
