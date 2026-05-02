//! SSH remote backend via system `ssh` CLI.
//!
//! Enabled with `--features ssh`.
//! Without the feature, all methods return an error instructing the user to rebuild.

#![allow(dead_code)] // feature-gated module

use crate::error::{Error, Result};
use crate::remote::{Remote, RemoteEntry};
use std::path::Path;

pub struct SshRemote {
    pub host: String,
    pub user: Option<String>,
    pub path: String,
}

impl SshRemote {
    #[allow(dead_code)]
    fn user_host(&self) -> String {
        match &self.user {
            Some(u) => format!("{u}@{}", self.host),
            None => self.host.clone(),
        }
    }
}

impl Remote for SshRemote {
    #[cfg(feature = "ssh")]
    fn push(&self, local: &Path, name: &str) -> Result<()> {
        let data = std::fs::read(local)
            .map_err(|e| Error::Remote(format!("read: {e}")))?;
        let remote_path = format!("{}/{}", self.path.trim_end_matches('/'), name);

        let status = std::process::Command::new("ssh")
            .arg("-o")
            .arg("BatchMode=no")
            .arg(self.user_host())
            .arg(format!("cat > {remote_path}"))
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| Error::Remote(format!("ssh spawn: {e}")))
            .and_then(|mut child| {
                use std::io::Write;
                if let Some(ref mut stdin) = child.stdin {
                    stdin.write_all(&data).ok();
                }
                child.wait().map_err(|e| Error::Remote(format!("ssh wait: {e}")))
            })?;

        if !status.success() {
            return Err(Error::Remote(format!("ssh push failed: exit {status}")));
        }

        println!("Pushed {name} to {}", self.user_host());
        Ok(())
    }

    #[cfg(not(feature = "ssh"))]
    fn push(&self, _local: &Path, _name: &str) -> Result<()> {
        Err(Error::Remote(
            "SSH support not compiled. Rebuild with --features ssh".into(),
        ))
    }

    #[cfg(feature = "ssh")]
    fn pull(&self, name: &str, local: &Path) -> Result<()> {
        let remote_path = format!("{}/{}", self.path.trim_end_matches('/'), name);

        let output = std::process::Command::new("ssh")
            .arg("-o")
            .arg("BatchMode=no")
            .arg(self.user_host())
            .arg(format!("cat {remote_path}"))
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .map_err(|e| Error::Remote(format!("ssh: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Remote(format!("ssh pull failed: {stderr}")));
        }

        let tmp = local.with_extension("ji_tmp");
        std::fs::write(&tmp, &output.stdout)
            .map_err(|e| Error::Remote(format!("write: {e}")))?;
        std::fs::rename(&tmp, local)
            .map_err(|e| Error::Remote(format!("rename: {e}")))?;

        println!("Pulled {name} from {}", self.user_host());
        Ok(())
    }

    #[cfg(not(feature = "ssh"))]
    fn pull(&self, _name: &str, _local: &Path) -> Result<()> {
        Err(Error::Remote(
            "SSH support not compiled. Rebuild with --features ssh".into(),
        ))
    }

    #[cfg(feature = "ssh")]
    fn list(&self) -> Result<Vec<RemoteEntry>> {
        let remote_path = self.path.trim_end_matches('/');
        let script = format!(
            "for f in {remote_path}/*; do [ -f \"$f\" ] && basename \"$f\"; done"
        );

        let output = std::process::Command::new("ssh")
            .arg("-o")
            .arg("BatchMode=no")
            .arg(self.user_host())
            .arg(&script)
            .output()
            .map_err(|e| Error::Remote(format!("ssh: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Remote(format!("ssh list failed: {stderr}")));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut entries = Vec::new();
        for line in stdout.lines() {
            let name = line.trim().to_string();
            if !name.is_empty() {
                entries.push(RemoteEntry {
                    name,
                    size: 0,
                    modified: chrono::Utc::now(),
                });
            }
        }
        Ok(entries)
    }

    #[cfg(not(feature = "ssh"))]
    fn list(&self) -> Result<Vec<RemoteEntry>> {
        Err(Error::Remote("SSH support not compiled. Rebuild with --features ssh".into()))
    }

    #[cfg(feature = "ssh")]
    fn delete(&self, name: &str) -> Result<()> {
        let remote_path = format!("{}/{}", self.path.trim_end_matches('/'), name);

        let status = std::process::Command::new("ssh")
            .arg("-o")
            .arg("BatchMode=no")
            .arg(self.user_host())
            .arg(format!("rm {remote_path}"))
            .status()
            .map_err(|e| Error::Remote(format!("ssh: {e}")))?;

        if !status.success() {
            return Err(Error::Remote(format!("ssh delete failed: exit {status}")));
        }
        println!("Deleted {name}");
        Ok(())
    }

    #[cfg(not(feature = "ssh"))]
    fn delete(&self, _name: &str) -> Result<()> {
        Err(Error::Remote("SSH support not compiled. Rebuild with --features ssh".into()))
    }

    #[cfg(feature = "ssh")]
    fn test(&self) -> Result<()> {
        let status = std::process::Command::new("ssh")
            .arg("-o")
            .arg("BatchMode=no")
            .arg("-o")
            .arg("ConnectTimeout=5")
            .arg(self.user_host())
            .arg("echo ok")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|e| Error::Remote(format!("ssh connect: {e}")))?;

        if status.success() {
            println!("Connection to {} OK", self.user_host());
            Ok(())
        } else {
            Err(Error::Remote(format!(
                "ssh connection failed: exit {status}"
            )))
        }
    }

    #[cfg(not(feature = "ssh"))]
    fn test(&self) -> Result<()> {
        Err(Error::Remote(
            "SSH support not compiled. Rebuild with --features ssh".into(),
        ))
    }
}
