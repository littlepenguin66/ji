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

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn default_excludes_ssh() {
        let _guard = path::TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        crate::store::path::with_test_home(tmp.path(), || {

        assert!(is_ignored(".ssh/config"));
        assert!(is_ignored(".ssh"));
        assert!(!is_ignored(".zshrc"));

        });
    }

    #[test]
    fn default_excludes_ds_store() {
        let _guard = path::TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        crate::store::path::with_test_home(tmp.path(), || {

        assert!(is_ignored(".DS_Store"));
        assert!(is_ignored("some/dir/.DS_Store"));
        assert!(!is_ignored(".gitconfig"));

        });
    }

    #[test]
    fn default_excludes_node_modules() {
        let _guard = path::TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        crate::store::path::with_test_home(tmp.path(), || {

        assert!(is_ignored("node_modules/react/index.js"));
        assert!(is_ignored("node_modules"));
        assert!(!is_ignored("src/node_modules_helper"));

        });
    }

    #[test]
    fn jiignore_file_patterns() {
        let _guard = path::TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        crate::store::path::with_test_home(tmp.path(), || {

        std::fs::create_dir_all(path::config_dir()).unwrap();
        std::fs::write(path::jiignore(), "*.zwc\n*.tmp\n").unwrap();

        assert!(is_ignored("test.zwc"));
        assert!(is_ignored("backup.tmp"));
        assert!(!is_ignored("test.conf"));

        });
    }

    #[test]
    fn jiignore_comments_and_blanks_ignored() {
        let _guard = path::TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        crate::store::path::with_test_home(tmp.path(), || {

        std::fs::create_dir_all(path::config_dir()).unwrap();
        std::fs::write(path::jiignore(), "# comment\n\n*.secret\n").unwrap();

        assert!(is_ignored("my.secret"));
        assert!(!is_ignored("# comment"));

        });
    }
}
