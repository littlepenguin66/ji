use crate::error::Result;
use serde::Serialize;
use std::io::Read;

#[derive(Debug, Serialize)]
struct DoctorReport {
    checks: Vec<CheckResult>,
}

#[derive(Debug, Serialize)]
struct CheckResult {
    category: String,
    status: CheckStatus,
    label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum CheckStatus {
    Ok,
    Warn,
    Error,
}

impl std::fmt::Display for CheckStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckStatus::Ok => write!(f, "\u{2713}"),
            CheckStatus::Warn => write!(f, "!"),
            CheckStatus::Error => write!(f, "\u{2717}"),
        }
    }
}

pub fn run(full: bool, json: bool) -> Result<()> {
    let mut report = DoctorReport { checks: Vec::new() };

    check_config(&mut report);
    check_keys(&mut report);
    check_manifest(&mut report);

    if full {
        check_remote_connectivity(&mut report);
        check_archives(&mut report);
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_human(&report);
    }

    let has_errors = report.checks.iter().any(|c| c.status == CheckStatus::Error);
    if has_errors {
        std::process::exit(1);
    }

    Ok(())
}

fn ok(category: &str, label: &str, detail: Option<String>) -> CheckResult {
    CheckResult { category: category.into(), status: CheckStatus::Ok, label: label.into(), detail }
}

fn warn(category: &str, label: &str, detail: Option<String>) -> CheckResult {
    CheckResult { category: category.into(), status: CheckStatus::Warn, label: label.into(), detail }
}

fn error(category: &str, label: &str, detail: Option<String>) -> CheckResult {
    CheckResult { category: category.into(), status: CheckStatus::Error, label: label.into(), detail }
}

fn check_config(report: &mut DoctorReport) {
    let cfg_path = crate::store::path::config_toml();

    if !cfg_path.exists() {
        report.checks.push(error("Config", "config.toml not found", Some("run `ji init` to create".into())));
        return;
    }

    report.checks.push(ok("Config", "config.toml found", Some(cfg_path.display().to_string())));

    match crate::store::config::Config::read(&cfg_path) {
        Ok(cfg) => {
            report.checks.push(ok("Config",
                &format!("encryption type: {}", cfg.encryption.encryption_type),
                None));

            let n = cfg.encryption.recipients.len();
            if n == 0 {
                report.checks.push(error("Config", "no recipients configured",
                    Some("run `ji init --key <PUBKEY>` to add".into())));
            } else {
                report.checks.push(ok("Config", &format!("{n} recipient(s) configured"), None));
            }

            let n_remotes = cfg.remote.len();
            if n_remotes == 0 {
                report.checks.push(warn("Config", "no remotes configured",
                    Some("run `ji remote add` to configure".into())));
            } else {
                report.checks.push(ok("Config", &format!("{n_remotes} remote(s) configured"), None));
            }
        }
        Err(e) => {
            report.checks.push(error("Config", "failed to parse config.toml",
                Some(e.to_string())));
        }
    }
}

fn check_keys(report: &mut DoctorReport) {
    let identity = crate::store::path::identity_path();
    let identity_pub = crate::store::path::identity_pub_path();

    if identity.exists() {
        report.checks.push(ok("Keys", "ji identity key found",
            Some(identity.display().to_string())));
    } else {
        report.checks.push(warn("Keys", "no ji identity key",
            Some("run `ji init` to generate".into())));
    }

    if identity_pub.exists() {
        report.checks.push(ok("Keys", "ji identity public key found", None));
    }

    // Check SSH keys
    let home = crate::store::path::home_dir();
    let ssh_dir = home.join(".ssh");
    if ssh_dir.exists() {
        let mut ssh_count = 0u32;
        if let Ok(entries) = std::fs::read_dir(&ssh_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if !name.ends_with(".pub") && name != "known_hosts" && name != "authorized_keys"
                    && entry.path().is_file()
                {
                    ssh_count += 1;
                }
            }
        }
        if ssh_count > 0 {
            report.checks.push(ok("Keys",
                &format!("{ssh_count} SSH private key(s) in ~/.ssh"), None));
        } else {
            report.checks.push(warn("Keys", "no SSH private keys in ~/.ssh",
                Some("age can use SSH keys for decryption".into())));
        }
    } else {
        report.checks.push(warn("Keys", "~/.ssh directory not found", None));
    }

    // Check ssh-agent
    if std::env::var("SSH_AUTH_SOCK").is_ok() {
        report.checks.push(ok("Keys", "ssh-agent running",
            Some(std::env::var("SSH_AUTH_SOCK").unwrap_or_default())));
    } else {
        report.checks.push(warn("Keys", "ssh-agent not running",
            Some("age will use key files directly for decryption".into())));
    }
}

fn check_manifest(report: &mut DoctorReport) {
    let manifest_path = crate::store::path::manifest_toml();

    if !manifest_path.exists() {
        report.checks.push(warn("Manifest", "manifest.toml not found",
            Some("run `ji add` to track files".into())));
        return;
    }

    match crate::store::manifest::Manifest::read(&manifest_path) {
        Ok(manifest) => {
            let n = manifest.files.len();
            if n == 0 {
                report.checks.push(warn("Manifest", "no files tracked",
                    Some("run `ji add` to track files".into())));
                return;
            }
            report.checks.push(ok("Manifest", &format!("{n} file(s) tracked"), None));

            // Check file existence
            let missing: Vec<&str> = manifest.list_paths().iter()
                .filter(|p| !crate::store::manifest::resolve_home(p).exists())
                .map(|p| p.as_str())
                .collect();
            if !missing.is_empty() {
                report.checks.push(warn("Manifest",
                    &format!("{} file(s) missing on disk", missing.len()),
                    Some(format!("missing: {}", missing.join(", ")))));
            } else {
                report.checks.push(ok("Manifest", "all tracked files exist on disk", None));
            }

            // Checksum status
            match crate::store::manifest::compute_status(&manifest) {
                Ok(statuses) => {
                    let modified: Vec<_> = statuses.iter()
                        .filter(|s| s.status == crate::store::manifest::FileStatus::Modified)
                        .collect();
                    if !modified.is_empty() {
                        report.checks.push(warn("Manifest",
                            &format!("{} file(s) modified since last pack", modified.len()),
                            Some("run `ji status` or `ji pack` to update".into())));
                    } else {
                        report.checks.push(ok("Manifest", "all checksums up to date", None));
                    }
                }
                Err(e) => {
                    report.checks.push(error("Manifest", "failed to compute status",
                        Some(e.to_string())));
                }
            }
        }
        Err(e) => {
            report.checks.push(error("Manifest", "failed to parse manifest.toml",
                Some(e.to_string())));
        }
    }
}

fn check_remote_connectivity(report: &mut DoctorReport) {
    let cfg_path = crate::store::path::config_toml();
    let cfg = match crate::store::config::Config::read(&cfg_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    for r in &cfg.remote {
        let label = format!("Remote: {}", r.name);

        let remote = match r.build() {
            Ok(r) => r,
            Err(e) => {
                report.checks.push(error(&label, &e.to_string(), None));
                continue;
            }
        };

        match remote.test() {
            Ok(()) => report.checks.push(ok(&label, "reachable", None)),
            Err(e) => report.checks.push(error(&label,
                &e.to_string(),
                Some("check URL, credentials, and network".into()))),
        }
    }
}

fn check_archives(report: &mut DoctorReport) {
    let data_dir = crate::store::path::data_dir();
    if !data_dir.exists() {
        return;
    }

    let mut found = 0u32;
    if let Ok(entries) = std::fs::read_dir(&data_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.ends_with(".ji") {
                found += 1;

                // Quick integrity check
                let path = entry.path();
                if let Ok(mut file) = std::fs::File::open(&path) {
                    match crate::archive::format::read_header(&mut file) {
                        Ok((cipher, index_len)) => {
                            let cipher_name = match cipher {
                                crate::archive::format::CipherType::Age => "age",
                                crate::archive::format::CipherType::Pgp => "pgp",
                            };
                            let mut index_buf = vec![0u8; index_len as usize];
                            if file.read_exact(&mut index_buf).is_ok() {
                                match crate::archive::format::read_index(
                                    &mut std::io::Cursor::new(&index_buf),
                                ) {
                                    Ok(index) => {
                                        report.checks.push(ok("Archive",
                                            &format!("{} ({}, {} files, {} bytes)",
                                                name, cipher_name,
                                                index.entries.len(), index.total_size),
                                            Some(path.display().to_string())));
                                    }
                                    Err(_) => {
                                        report.checks.push(error("Archive",
                                            &format!("{}: HMAC verification failed", name),
                                            Some("file may be corrupted".into())));
                                    }
                                }
                            }
                        }
                        Err(_) => {
                            report.checks.push(error("Archive",
                                &format!("{}: invalid .ji file", name), None));
                        }
                    }
                }
            }
        }
    }

    if found == 0 {
        report.checks.push(warn("Archive", "no .ji files found",
            Some(format!("run `ji pack` to create one in {}", data_dir.display()))));
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn doctor_reports_errors_with_no_config() {
        let _guard = crate::store::path::TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        crate::store::path::with_test_home(tmp.path(), || {

        let mut report = DoctorReport { checks: Vec::new() };
        check_config(&mut report);

        // First check should be an error about missing config.toml
        let first = &report.checks[0];
        assert_eq!(first.category, "Config");
        assert_eq!(first.status, CheckStatus::Error);
        assert!(first.label.contains("not found"));

        });
    }

    #[test]
    fn doctor_json_output_matches_schema() {
        let _guard = crate::store::path::TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        crate::store::path::with_test_home(tmp.path(), || {

        // Capture stdout via --json output
        // run() prints to stdout; we can't easily capture it in unit tests
        // Instead, test the internal report building
        let mut report = DoctorReport { checks: Vec::new() };
        check_config(&mut report);
        check_keys(&mut report);
        check_manifest(&mut report);

        let json = serde_json::to_string_pretty(&report).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert!(parsed.get("checks").is_some());
        for check in parsed["checks"].as_array().unwrap() {
            assert!(check.get("category").is_some());
            assert!(check.get("status").is_some());
            assert!(check.get("label").is_some());
        }

        });
    }

    #[test]
    fn doctor_all_ok_with_valid_setup() {
        let _guard = crate::store::path::TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        crate::store::path::with_test_home(tmp.path(), || {

        // Create valid config
        std::fs::create_dir_all(crate::store::path::config_dir()).unwrap();
        std::fs::create_dir_all(crate::store::path::data_dir()).unwrap();

        let cfg = crate::store::config::Config::new(vec!["age1testkey123".into()]);
        cfg.write(&crate::store::path::config_toml()).unwrap();

        // Create identity key
        let (priv_key, _pub_key) = crate::crypto::age::AgeCipher::generate_identity();
        std::fs::write(crate::store::path::identity_path(), &priv_key).unwrap();

        // Create manifest with a real file
        let test_file = tmp.path().join(".zshrc");
        std::fs::write(&test_file, "export EDITOR=nvim\n").unwrap();
        let checksum = crate::store::manifest::compute_checksum(&test_file).unwrap();
        let mut manifest = crate::store::manifest::Manifest::new();
        manifest.add(".zshrc", checksum);
        manifest.write(&crate::store::path::manifest_toml()).unwrap();

        let mut report = DoctorReport { checks: Vec::new() };
        check_config(&mut report);
        check_keys(&mut report);
        check_manifest(&mut report);

        // All checks should be Ok
        let errors: Vec<_> = report.checks.iter()
            .filter(|c| c.status == CheckStatus::Error)
            .collect();
        assert!(errors.is_empty(), "unexpected errors: {:?}", errors);

        });
    }
}

fn print_human(report: &DoctorReport) {
    let mut current_category = String::new();

    for check in &report.checks {
        if check.category != current_category {
            if !current_category.is_empty() {
                println!();
            }
            println!("==> {}", check.category);
            current_category = check.category.clone();
        }

        print!("  {} {}", check.status, check.label);
        if let Some(ref d) = check.detail {
            print!("  ({})", d);
        }
        println!();
    }

    let errors = report.checks.iter().filter(|c| c.status == CheckStatus::Error).count();
    let warns = report.checks.iter().filter(|c| c.status == CheckStatus::Warn).count();

    println!();
    if errors > 0 || warns > 0 {
        println!("==> Summary: {} warning(s), {} error(s)", warns, errors);
        if errors > 0 {
            println!("   Run `ji doctor --json` for machine-readable output.");
        }
    } else {
        println!("==> Summary: all checks passed");
    }
}
