use crate::store::path;

/// Check if a path should be ignored based on .jiignore rules.
/// Default excludes: .ssh/, .DS_Store, node_modules/
pub fn is_ignored(relative_path: &str) -> bool {
    // Default exclusions
    if relative_path.starts_with(".ssh/") || relative_path == ".ssh" {
        return true;
    }
    if relative_path == ".DS_Store" || relative_path.ends_with("/.DS_Store") {
        return true;
    }
    if relative_path.starts_with("node_modules/") || relative_path == "node_modules" {
        return true;
    }

    // Check .jiignore file
    let jiignore = path::jiignore();
    if jiignore.exists() {
        if let Ok(contents) = std::fs::read_to_string(&jiignore) {
            for line in contents.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if match_simple_pattern(line, relative_path) {
                    return true;
                }
            }
        }
    }

    false
}

fn match_simple_pattern(pattern: &str, path: &str) -> bool {
    if pattern == path {
        return true;
    }
    if pattern.ends_with('/') && path.starts_with(pattern) {
        return true;
    }
    if let Ok(glob_pat) = glob::Pattern::new(pattern) {
        return glob_pat.matches(path);
    }
    false
}
