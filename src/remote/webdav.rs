use crate::error::{Error, Result};
use crate::remote::{Remote, RemoteEntry};
use reqwest::blocking::Client;
use std::path::Path;

pub struct WebdavRemote {
    pub url: String,
    pub user: Option<String>,
}

impl WebdavRemote {
    fn client(&self) -> Result<Client> {
        Client::builder()
            .build()
            .map_err(|e| Error::Remote(format!("http client: {e}")))
    }

    fn credentials(&self) -> Result<(String, Option<String>)> {
        let user = self
            .user
            .clone()
            .ok_or_else(|| Error::Remote("no user configured".into()))?;
        let password =
            rpassword::prompt_password(format!("Password for {user}@{url}: ", url = self.url))
                .map_err(|e| Error::Remote(format!("password: {e}")))?;
        Ok((user, Some(password)))
    }

    fn base_url(&self) -> String {
        self.url.trim_end_matches('/').to_string()
    }
}

impl Remote for WebdavRemote {
    fn push(&self, local: &Path, name: &str) -> Result<()> {
        let (user, password) = self.credentials()?;
        let data = std::fs::read(local).map_err(|e| Error::Remote(format!("read: {e}")))?;
        let url = format!("{}/{}", self.base_url(), name);

        let resp = self
            .client()?
            .put(&url)
            .basic_auth(&user, password)
            .body(data)
            .send()
            .map_err(|e| Error::Remote(format!("push: {e}")))?;

        if !resp.status().is_success() {
            return Err(Error::Remote(format!("push failed: {}", resp.status())));
        }

        println!("Pushed {name} to {}", self.url);
        Ok(())
    }

    fn pull(&self, name: &str, local: &Path) -> Result<()> {
        let url = format!("{}/{}", self.base_url(), name);

        let resp = self
            .client()?
            .get(&url)
            .send()
            .map_err(|e| Error::Remote(format!("pull: {e}")))?;

        if !resp.status().is_success() {
            return Err(Error::Remote(format!("pull failed: {}", resp.status())));
        }

        let data = resp
            .bytes()
            .map_err(|e| Error::Remote(format!("pull read: {e}")))?;

        let tmp = local.with_extension("ji_tmp");
        std::fs::write(&tmp, &data).map_err(|e| Error::Remote(format!("write: {e}")))?;
        std::fs::rename(&tmp, local).map_err(|e| Error::Remote(format!("rename: {e}")))?;

        println!("Pulled {name} from {}", self.url);
        Ok(())
    }

    fn list(&self) -> Result<Vec<RemoteEntry>> {
        let url = self.base_url();
        let resp = self
            .client()?
            .request(reqwest::Method::from_bytes(b"PROPFIND").unwrap(), &url)
            .send()
            .map_err(|e| Error::Remote(format!("list: {e}")))?;

        let body = resp
            .text()
            .map_err(|e| Error::Remote(format!("list read: {e}")))?;

        parse_propfind(&body)
    }

    fn delete(&self, name: &str) -> Result<()> {
        let (user, password) = self.credentials()?;
        let url = format!("{}/{}", self.base_url(), name);

        let resp = self
            .client()?
            .delete(&url)
            .basic_auth(&user, password)
            .send()
            .map_err(|e| Error::Remote(format!("delete: {e}")))?;

        if !resp.status().is_success() {
            return Err(Error::Remote(format!("delete failed: {}", resp.status())));
        }

        println!("Deleted {name}");
        Ok(())
    }

    fn test(&self) -> Result<()> {
        let url = self.base_url();
        let resp = self
            .client()?
            .head(&url)
            .send()
            .map_err(|e| Error::Remote(format!("test: {e}")))?;

        if resp.status().is_success() || resp.status().as_u16() == 401 {
            println!("Connection to {} OK (status: {})", self.url, resp.status());
            Ok(())
        } else {
            Err(Error::Remote(format!("test failed: {}", resp.status())))
        }
    }
}

fn parse_propfind(xml: &str) -> Result<Vec<RemoteEntry>> {
    let mut entries = Vec::new();

    for line in xml.lines() {
        if line.contains("<D:href>") || line.contains("<d:href>") {
            let start = line.find('>').map(|i| i + 1).unwrap_or(0);
            let end = line.rfind('<').unwrap_or(line.len());
            let href = line[start..end].trim();
            if href != "/" && !href.is_empty() {
                let name = href
                    .trim_start_matches('/')
                    .trim_end_matches('/')
                    .to_string();
                if !name.is_empty() {
                    entries.push(RemoteEntry {
                        name,
                        size: 0,
                        modified: chrono::Utc::now(),
                    });
                }
            }
        }
    }

    Ok(entries)
}
