use crate::error::Result;
use crate::store::config::Config;
use crate::store::manifest::Manifest;
use crate::store::path;

pub fn run(remote_name: String) -> Result<()> {
    let config = Config::read(&path::config_toml())?;
    let _r = config
        .remote
        .iter()
        .find(|r| r.name == remote_name)
        .ok_or_else(|| {
            crate::error::Error::Remote(format!("remote '{remote_name}' not found"))
        })?;

    let manifest = Manifest::read(&path::manifest_toml())?;

    if manifest.files.is_empty() && !path::manifest_toml().exists() {
        println!("First use: pulling from remote...");
        return crate::commands::pull::run(remote_name);
    }

    if !manifest.files.is_empty() {
        let statuses = crate::store::manifest::compute_status(&manifest)?;
        let has_changes = statuses
            .iter()
            .any(|e| e.status != crate::store::manifest::FileStatus::Unchanged);

        if has_changes {
            println!("Local changes detected, packing...");
            let host = hostname::get()
                .map(|h| h.to_string_lossy().to_string())
                .unwrap_or_else(|_| "unknown".into());
            let output = path::default_output(&host);
            crate::commands::pack::run(Some(output.clone()), false, false)?;
            return crate::commands::push::run(remote_name.clone(), output);
        }
    }

    println!("Pulling latest from remote...");
    crate::commands::pull::run(remote_name)
}
