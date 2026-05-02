use crate::error::Result;
use crate::store::config::Config;
use crate::store::path;

pub fn run(name: String) -> Result<()> {
    let config = Config::read(&path::config_toml())?;
    let r = config
        .remote
        .iter()
        .find(|r| r.name == name)
        .ok_or_else(|| crate::error::Error::Remote(format!("remote '{name}' not found")))?;

    std::fs::create_dir_all(path::data_dir()).map_err(|e| crate::error::Error::Io(e))?;
    let host = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "default".into());
    let output = path::data_dir().join(format!("{host}.ji"));

    let remote = r.build()?;
    remote.pull(&format!("{host}.ji"), &output)
}
