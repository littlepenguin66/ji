use crate::error::Result;
use crate::remote::webdav::WebdavRemote;
use crate::remote::Remote;
use crate::store::config::Config;
use crate::store::path;

pub fn run(name: String) -> Result<()> {
    let config = Config::read(&path::config_toml())?;
    let r = config
        .remote
        .iter()
        .find(|r| r.name == name)
        .ok_or_else(|| crate::error::Error::Remote(format!("remote '{name}' not found")))?;

    // Pull to ~/.local/share/ji/
    std::fs::create_dir_all(path::data_dir()).map_err(|e| crate::error::Error::Io(e))?;
    let host = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "default".into());
    let output = path::data_dir().join(format!("{host}.ji"));

    match r.remote_type.as_str() {
        "webdav" => {
            let remote = WebdavRemote {
                url: r.url.clone(),
                user: r.user.clone(),
            };
            remote.pull(&format!("{host}.ji"), &output)
        }
        "ssh" => {
            let (ssh_host, path) = crate::remote::parse_ssh_url(&r.url)?;
            let remote = crate::remote::ssh::SshRemote {
                host: ssh_host,
                user: r.user.clone(),
                path,
            };
            remote.pull(&format!("{host}.ji"), &output)
        }
        _ => Err(crate::error::Error::Remote(format!(
            "unsupported remote type: {}",
            r.remote_type
        ))),
    }
}
