use crate::error::Result;
use crate::store::manifest::{self, Manifest};
use crate::store::path;
use similar::{ChangeTag, TextDiff};
use std::path::PathBuf;

pub fn run(path_filter: Option<PathBuf>) -> Result<()> {
    let manifest = Manifest::read(&path::manifest_toml())?;

    if manifest.files.is_empty() {
        println!("(no files tracked)");
        return Ok(());
    }

    let cache_dir = path::cache_dir();
    let mut has_diff = false;

    for rel_path in manifest.list_paths() {
        if let Some(ref filter) = path_filter {
            if *rel_path != filter.to_string_lossy() {
                continue;
            }
        }

        let abs = manifest::resolve_home(rel_path);
        let cache_path = cache_dir.join(rel_path);

        if !abs.exists() && !cache_path.exists() {
            continue;
        }

        if !abs.exists() {
            println!("--- a/{}", rel_path);
            println!("+++ /dev/null");
            println!("@@ -1 +0,0 @@");
            println!("-<deleted>");
            has_diff = true;
            continue;
        }

        if !cache_path.exists() {
            println!("--- /dev/null");
            println!("+++ b/{}", rel_path);
            println!("@@ -0,0 +1 @@");
            println!("+<new file>");
            has_diff = true;
            continue;
        }

        let current = std::fs::read(&abs).ok();
        let cached = std::fs::read(&cache_path).ok();

        match (current, cached) {
            (Some(curr), Some(cached)) => {
                if curr == cached {
                    continue;
                }

                if is_binary(&curr) || is_binary(&cached) {
                    println!("Binary files differ: {rel_path}");
                    has_diff = true;
                    continue;
                }

                let old = String::from_utf8_lossy(&cached).to_string();
                let new = String::from_utf8_lossy(&curr).to_string();
                let diff = TextDiff::from_lines(&old, &new);

                println!("--- a/{}", rel_path);
                println!("+++ b/{}", rel_path);

                for change in diff.iter_all_changes() {
                    match change.tag() {
                        ChangeTag::Equal => {
                            for line in change.value().lines() {
                                println!(" {line}");
                            }
                        }
                        ChangeTag::Delete => {
                            for line in change.value().lines() {
                                println!("-{line}");
                            }
                        }
                        ChangeTag::Insert => {
                            for line in change.value().lines() {
                                println!("+{line}");
                            }
                        }
                    }
                }
                has_diff = true;
            }
            _ => {}
        }
    }

    if !has_diff {
        println!("(no changes)");
    }

    Ok(())
}

fn is_binary(data: &[u8]) -> bool {
    data.iter().take(1024).any(|&b| b == 0)
}
