use crate::error::Result;
use crate::remote::webdav::WebdavRemote;
use crate::remote::Remote;
use crate::store::config::Config;
use crate::store::path;
use std::path::PathBuf;

pub fn run(name: String, input: PathBuf) -> Result<()> {
    // Resolve input path if not absolute
    let input = if input.is_absolute() || input.exists() {
        input
    } else {
        // Auto-lookup in ~/.local/share/ji/
        let auto = path::data_dir().join(&input);
        if auto.exists() {
            auto
        } else {
            return Err(crate::error::Error::Archive(format!(
                "file not found: {}",
                input.display()
            )));
        }
    };

    if !input.exists() {
        return Err(crate::error::Error::Archive(format!(
            "file not found: {}",
            input.display()
        )));
    }

    let config = Config::read(&path::config_toml())?;
    let r = config
        .remote
        .iter()
        .find(|r| r.name == name)
        .ok_or_else(|| crate::error::Error::Remote(format!("remote '{name}' not found")))?;

    let file_name = input
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown.ji".into());

    match r.remote_type.as_str() {
        "webdav" => {
            let remote = WebdavRemote {
                url: r.url.clone(),
                user: r.user.clone(),
            };
            remote.push(&input, &file_name)
        }
        "ssh" => {
            let (host, path) = crate::remote::parse_ssh_url(&r.url)?;
            let remote = crate::remote::ssh::SshRemote {
                host,
                user: r.user.clone(),
                path,
            };
            remote.push(&input, &file_name)
        }
        _ => Err(crate::error::Error::Remote(format!(
            "unsupported remote type: {}",
            r.remote_type
        ))),
    }
}
