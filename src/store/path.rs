use std::path::PathBuf;
use std::sync::Mutex;

#[allow(dead_code)]
pub static TEST_MUTEX: Mutex<()> = Mutex::new(());

#[allow(dead_code)]
pub fn with_test_home(dir: &std::path::Path, f: impl FnOnce()) {
    unsafe { std::env::set_var("JI_TEST_HOME", dir.as_os_str()) };
    f();
    unsafe { std::env::remove_var("JI_TEST_HOME") };
}

pub fn home_dir() -> PathBuf {
    if let Ok(test_home) = std::env::var("JI_TEST_HOME") {
        PathBuf::from(test_home)
    } else {
        std::env::var("HOME")
            .map(PathBuf::from)
            .expect("HOME not set")
    }
}

fn base_config_dir() -> PathBuf {
    home_dir().join(".config")
}

fn base_data_dir() -> PathBuf {
    home_dir().join(".local").join("share")
}

pub fn config_dir() -> PathBuf {
    base_config_dir().join("ji")
}

pub fn data_dir() -> PathBuf {
    base_data_dir().join("ji")
}

pub fn config_toml() -> PathBuf {
    config_dir().join("config.toml")
}

pub fn manifest_toml() -> PathBuf {
    config_dir().join("manifest.toml")
}

pub fn jiignore() -> PathBuf {
    config_dir().join(".jiignore")
}

pub fn identity_path() -> PathBuf {
    data_dir().join("ji.identity.age")
}

pub fn identity_pub_path() -> PathBuf {
    data_dir().join("ji.identity.age.pub")
}

pub fn cache_dir() -> PathBuf {
    data_dir().join("cache")
}

pub fn default_output(hostname: &str) -> PathBuf {
    data_dir().join(format!("{}.ji", hostname))
}

pub fn discover_ji() -> crate::error::Result<PathBuf> {
    let dir = data_dir();
    let mut candidates: Vec<_> = std::fs::read_dir(&dir)
        .map_err(|_| {
            crate::error::Error::Archive(format!("no .ji files found in {}", dir.display()))
        })?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "ji"))
        .filter_map(|e| {
            let meta = e.metadata().ok()?;
            Some((e.path(), meta.modified().ok()?))
        })
        .collect();
    candidates.sort_by(|a, b| b.1.cmp(&a.1));
    candidates.first().map(|(p, _)| p.clone()).ok_or_else(|| {
        crate::error::Error::Archive(format!("no .ji files found in {}", dir.display()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_home_overrides_paths() {
        let _guard = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe { std::env::set_var("JI_TEST_HOME", tmp.path().as_os_str()) };

        let cfg = config_dir();
        assert!(cfg.starts_with(tmp.path()));

        let data = data_dir();
        assert!(data.starts_with(tmp.path()));

        unsafe { std::env::remove_var("JI_TEST_HOME") };
    }

    #[test]
    fn default_output_naming() {
        let _guard = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe { std::env::set_var("JI_TEST_HOME", tmp.path().as_os_str()) };

        let out = default_output("mbp");
        assert_eq!(out.file_name().unwrap(), "mbp.ji");

        unsafe { std::env::remove_var("JI_TEST_HOME") };
    }

    #[test]
    fn identity_paths() {
        let _guard = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        unsafe { std::env::set_var("JI_TEST_HOME", tmp.path().as_os_str()) };

        assert!(identity_path().ends_with("ji.identity.age"));
        assert!(identity_pub_path().ends_with("ji.identity.age.pub"));

        unsafe { std::env::remove_var("JI_TEST_HOME") };
    }
}
