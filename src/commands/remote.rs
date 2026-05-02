use crate::error::Result;
use crate::remote::webdav::WebdavRemote;
use crate::remote::Remote;
use crate::store::config::{Config, RemoteConfig};
use crate::store::path;

pub fn run_add(name: String, remote_type: String, url: String, user: Option<String>) -> Result<()> {
    let config_path = path::config_toml();
    let mut config = Config::read(&config_path)?;

    // Check for duplicate name
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
    println!("Added remote '{name}' ({remote_type}: {url})");
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
    config.write(&config_path)?;
    println!("Removed remote '{name}'");
    Ok(())
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

    match r.remote_type.as_str() {
        "webdav" => {
            let remote = WebdavRemote {
                url: r.url.clone(),
                user: r.user.clone(),
            };
            remote.test()
        }
        "ssh" => {
            let (host, path) = crate::remote::parse_ssh_url(&r.url)?;
            let remote = crate::remote::ssh::SshRemote {
                host,
                user: r.user.clone(),
                path,
            };
            remote.test()
        }
        _ => Err(crate::error::Error::Remote(format!(
            "unsupported remote type: {}",
            r.remote_type
        ))),
    }
}

pub fn run_files(name: String) -> Result<()> {
    let config = Config::read(&path::config_toml())?;
    let r = config
        .remote
        .iter()
        .find(|r| r.name == name)
        .ok_or_else(|| crate::error::Error::Remote(format!("remote '{name}' not found")))?;

    let entries = match r.remote_type.as_str() {
        "webdav" => {
            let remote = WebdavRemote {
                url: r.url.clone(),
                user: r.user.clone(),
            };
            remote.list()
        }
        "ssh" => {
            let (host, path) = crate::remote::parse_ssh_url(&r.url)?;
            let remote = crate::remote::ssh::SshRemote {
                host,
                user: r.user.clone(),
                path,
            };
            remote.list()
        }
        _ => Err(crate::error::Error::Remote(format!(
            "unsupported remote type: {}",
            r.remote_type
        ))),
    }?;

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

    match r.remote_type.as_str() {
        "webdav" => {
            let remote = WebdavRemote {
                url: r.url.clone(),
                user: r.user.clone(),
            };
            remote.delete(file)
        }
        "ssh" => {
            let (host, path) = crate::remote::parse_ssh_url(&r.url)?;
            let remote = crate::remote::ssh::SshRemote {
                host,
                user: r.user.clone(),
                path,
            };
            remote.delete(file)
        }
        _ => Err(crate::error::Error::Remote(format!(
            "unsupported remote type: {}",
            r.remote_type
        ))),
    }
}
