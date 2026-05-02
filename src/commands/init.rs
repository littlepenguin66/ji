use crate::crypto::age::AgeCipher;
use crate::error::Result;
use crate::store::config::Config;
use crate::store::path;

pub fn run(keys: Vec<String>, auto: bool, force: bool) -> Result<()> {
    let config_path = path::config_toml();

    if config_path.exists() && !force {
        eprintln!(
            "config already exists at {} (use --force to overwrite)",
            config_path.display()
        );
        return Ok(());
    }

    std::fs::create_dir_all(path::config_dir()).map_err(|e| crate::error::Error::Io(e))?;
    std::fs::create_dir_all(path::data_dir()).map_err(|e| crate::error::Error::Io(e))?;

    let mut recipients: Vec<String> = keys;

    if recipients.is_empty() && !auto {
        eprintln!("No recipients specified. Generating age keypair...");
        auto_generate_keypair(&mut recipients)?;
    } else if auto {
        auto_generate_keypair(&mut recipients)?;
    }

    let config = Config::new(recipients);
    config.write(&config_path)?;

    println!("Config written to {}", config_path.display());
    println!();
    println!("Next: ji add your dotfiles, then ji pack");

    Ok(())
}

fn auto_generate_keypair(recipients: &mut Vec<String>) -> Result<()> {
    let (priv_key, pub_key) = AgeCipher::generate_identity();

    let identity_path = path::identity_path();
    let identity_pub_path = path::identity_pub_path();

    std::fs::create_dir_all(identity_path.parent().unwrap())
        .map_err(|e| crate::error::Error::Io(e))?;

    let tmp = identity_path.with_extension("tmp");
    std::fs::write(&tmp, &priv_key).map_err(|e| crate::error::Error::Io(e))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o600))
            .map_err(|e| crate::error::Error::Io(e))?;
    }

    std::fs::rename(&tmp, &identity_path).map_err(|e| crate::error::Error::Io(e))?;
    println!("Private key saved to {}", identity_path.display());

    std::fs::write(&identity_pub_path, format!("{}\n", pub_key))
        .map_err(|e| crate::error::Error::Io(e))?;
    println!("Public key saved to {}", identity_pub_path.display());

    recipients.push(pub_key);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn auto_generate_creates_keypair_and_files() {
        let _guard = crate::store::path::TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        crate::store::path::with_test_home(tmp.path(), || {

        let mut recipients = vec![];
        auto_generate_keypair(&mut recipients).expect("generate keypair");
        assert_eq!(recipients.len(), 1);
        assert!(recipients[0].starts_with("age1"));

        assert!(path::identity_path().exists());
        assert!(path::identity_pub_path().exists());

        });
    }

    #[test]
    fn init_writes_config() {
        let _guard = crate::store::path::TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        crate::store::path::with_test_home(tmp.path(), || {

        run(vec!["age1testkey123".into()], false, false).expect("init");

        let config_path = path::config_toml();
        assert!(config_path.exists());
        let config = Config::read(&config_path).unwrap();
        assert_eq!(config.encryption.recipients, vec!["age1testkey123"]);

        });
    }

    #[test]
    fn init_skips_if_config_exists() {
        let _guard = crate::store::path::TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        crate::store::path::with_test_home(tmp.path(), || {

        run(vec!["age1first".into()], false, false).expect("first init");
        run(vec!["age1second".into()], false, false).expect("second init");

        let config = Config::read(&path::config_toml()).unwrap();
        assert_eq!(config.encryption.recipients, vec!["age1first"]);

        });
    }

    #[test]
    fn init_force_overwrites_config() {
        let _guard = crate::store::path::TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        crate::store::path::with_test_home(tmp.path(), || {

        run(vec!["age1first".into()], false, false).expect("first init");
        run(vec!["age1second".into()], false, true).expect("force init");

        let config = Config::read(&path::config_toml()).unwrap();
        assert_eq!(config.encryption.recipients, vec!["age1second"]);

        });
    }
}
