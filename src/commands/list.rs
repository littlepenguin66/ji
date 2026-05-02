use crate::error::Result;
use crate::store::manifest::Manifest;
use crate::store::path;
use std::collections::BTreeMap;

pub fn run(json: bool, verbose: bool) -> Result<()> {
    let manifest = Manifest::read(&path::manifest_toml())?;

    if json {
        let output = serde_json::to_string_pretty(&manifest.files)?;
        println!("{output}");
        return Ok(());
    }

    if manifest.files.is_empty() {
        println!("(no files tracked)");
        return Ok(());
    }

    let root = build_tree(manifest.list_paths(), &manifest, verbose);
    print_tree(&root, "", verbose);

    Ok(())
}

#[derive(Default)]
struct Node {
    children: BTreeMap<String, Node>,
    entry: Option<(u64, String)>,
}

fn build_tree(paths: Vec<&String>, manifest: &Manifest, verbose: bool) -> Node {
    let mut root = Node::default();
    for path in paths {
        let parts: Vec<&str> = path.split('/').collect();
        let mut current = &mut root;
        for (i, part) in parts.iter().enumerate() {
            current = current.children.entry(part.to_string()).or_default();
            if i == parts.len() - 1 {
                if verbose {
                    let home = crate::store::manifest::resolve_home(path);
                    let size = std::fs::metadata(&home).map(|m| m.len()).unwrap_or(0);
                    let short = manifest
                        .get(path)
                        .map(|e| e.checksum.chars().take(8).collect::<String>())
                        .unwrap_or_default();
                    current.entry = Some((size, short));
                } else {
                    current.entry = Some((0, String::new()));
                }
            }
        }
    }
    root
}

fn print_tree(node: &Node, prefix: &str, verbose: bool) {
    let entries: Vec<_> = node.children.iter().collect();

    for (i, (name, child)) in entries.iter().enumerate() {
        let is_last = i == entries.len() - 1;
        let branch = if is_last { "\u{2514}" } else { "\u{251c}" };
        let connector = if prefix.is_empty() {
            branch.to_string()
        } else {
            format!("{prefix}{branch}")
        };

        if child.children.is_empty() {
            if verbose {
                if let Some((size, checksum)) = &child.entry {
                    println!("{connector} {name}  {}  {}", human_size(*size), checksum);
                } else {
                    println!("{connector} {name}");
                }
            } else {
                println!("{connector} {name}");
            }
        } else {
            println!("{connector} {name}/");
            let new_prefix = if is_last {
                format!("{prefix}   ")
            } else {
                format!("{prefix}\u{2502}  ")
            };
            print_tree(child, &new_prefix, verbose);
        }
    }
}

fn human_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{:>5}B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:>5.1}K", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:>5.1}M", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:>5.1}G", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_json_output() {
        let _guard = crate::store::path::TEST_MUTEX
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        crate::store::path::with_test_home(tmp.path(), || {
            let mut m = Manifest::new();
            m.add(".zshrc", "abc123".into());
            m.write(&path::manifest_toml()).unwrap();
            run(true, false).expect("list --json");
        });
    }

    #[test]
    fn list_empty() {
        let _guard = crate::store::path::TEST_MUTEX
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        crate::store::path::with_test_home(tmp.path(), || {
            run(false, false).expect("list");
        });
    }

    #[test]
    fn list_tree_output() {
        let _guard = crate::store::path::TEST_MUTEX
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        crate::store::path::with_test_home(tmp.path(), || {
            let mut m = Manifest::new();
            m.add(".zshrc", "abc12345".into());
            m.add(".gitconfig", "def67890".into());
            m.write(&path::manifest_toml()).unwrap();
            run(false, false).expect("list");
        });
    }
}
