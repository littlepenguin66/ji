use crate::error::Result;
use crate::store::config::Config;
use crate::store::path;
use std::path::PathBuf;

pub fn run(name: String, input: PathBuf) -> Result<()> {
    let input = if input.is_absolute() || input.exists() {
        input
    } else {
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

    let remote = r.build()?;
    remote.push(&input, &file_name)
}
