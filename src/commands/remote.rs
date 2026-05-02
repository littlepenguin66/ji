use crate::error::Result;
use crate::store::config::{Config, RemoteConfig};
use crate::store::path;

pub fn run_add(name: String, remote_type: String, url: String, user: Option<String>) -> Result<()> {
    let config_path = path::config_toml();
    let mut config = Config::read(&config_path)?;

    if config.remote.iter().any(|r| r.name == name) {
        return Err(crate::error::Error::Remote(format!(
            "remote '{name}' already exists"
        )));
    }

    config.remote.push(RemoteConfig {
        name: name.clone(),
        remote_type: remote_type.clone(),
        url: url.clone(),
        user,
    });

    config.write(&config_path)?;
    Ok(())
}

pub fn run_remove(name: String) -> Result<()> {
    let config_path = path::config_toml();
    let mut config = Config::read(&config_path)?;

    let idx = config
        .remote
        .iter()
        .position(|r| r.name == name)
        .ok_or_else(|| crate::error::Error::Remote(format!("remote '{name}' not found")))?;

    config.remote.remove(idx);
    config.write(&config_path)
}

pub fn run_list(json: bool) -> Result<()> {
    let config = Config::read(&path::config_toml())?;

    if json {
        let output = serde_json::to_string_pretty(&config.remote)?;
        println!("{output}");
    } else {
        if config.remote.is_empty() {
            println!("(no remotes configured)");
            return Ok(());
        }
        for r in &config.remote {
            println!(
                "{}  {}  {}  {}",
                r.name,
                r.remote_type,
                r.url,
                r.user.as_deref().unwrap_or("-")
            );
        }
    }

    Ok(())
}

pub fn run_test(name: String) -> Result<()> {
    let config = Config::read(&path::config_toml())?;
    let r = config
        .remote
        .iter()
        .find(|r| r.name == name)
        .ok_or_else(|| crate::error::Error::Remote(format!("remote '{name}' not found")))?;

    r.build()?.test()
}

pub fn run_files(name: String) -> Result<()> {
    let config = Config::read(&path::config_toml())?;
    let r = config
        .remote
        .iter()
        .find(|r| r.name == name)
        .ok_or_else(|| crate::error::Error::Remote(format!("remote '{name}' not found")))?;

    let entries = r.build()?.list()?;

    if entries.is_empty() {
        println!("(no files)");
    } else {
        for e in &entries {
            println!(
                "{:>10}  {}  {}",
                e.size,
                e.name,
                e.modified.format("%Y-%m-%d %H:%M")
            );
        }
    }

    Ok(())
}

pub fn run_delete(name: String, file: &str) -> Result<()> {
    let config = Config::read(&path::config_toml())?;
    let r = config
        .remote
        .iter()
        .find(|r| r.name == name)
        .ok_or_else(|| crate::error::Error::Remote(format!("remote '{name}' not found")))?;

    r.build()?.delete(file)
}
