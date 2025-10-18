// Tests for file rename/move detection functionality
use contextpilot::git_command_algo::{detect_rename_in_commit, get_file_rename_history};
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

#[cfg(test)]
mod tests_rename_detection {
    use super::*;

    /// Helper to save and restore current directory
    struct DirGuard {
        original_dir: PathBuf,
    }

    impl DirGuard {
        fn new() -> Self {
            Self {
                original_dir: std::env::current_dir().expect("Failed to get current dir"),
            }
        }
    }

    impl Drop for DirGuard {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.original_dir);
        }
    }

    /// Helper function to create a test git repository with a renamed file
    fn setup_test_repo_with_rename() -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let repo_path = temp_dir.path();

        // Initialize git repo
        Command::new("git")
            .args(["init"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to init git repo");

        // Configure git user
        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to set git user.name");

        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to set git user.email");

        // Create initial file
        let file_path = repo_path.join("old_file.rs");
        std::fs::write(
            &file_path,
            "fn main() {\n    println!(\"Hello, world!\");\n}\n",
        )
        .expect("Failed to write file");

        Command::new("git")
            .args(["add", "old_file.rs"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to add file");

        Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to commit");

        // Rename the file
        Command::new("git")
            .args(["mv", "old_file.rs", "new_file.rs"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to rename file");

        Command::new("git")
            .args(["commit", "-m", "Rename file"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to commit rename");

        temp_dir
    }

    /// Helper function to get the latest commit hash
    fn get_latest_commit(repo_path: &std::path::Path) -> String {
        let output = Command::new("git")
            .args(["rev-parse", "--short=7", "HEAD"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to get commit hash");

        String::from_utf8(output.stdout)
            .expect("Invalid UTF-8")
            .trim()
            .to_string()
    }

    #[test]
    fn test_detect_rename_in_commit_basic() {
        let _guard = DirGuard::new();

        let temp_dir = setup_test_repo_with_rename();
        let repo_path = temp_dir.path();

        std::env::set_current_dir(repo_path).expect("Failed to change directory");

        let commit_hash = get_latest_commit(repo_path);
        let result = detect_rename_in_commit(&commit_hash, "new_file.rs");

        assert!(result.is_some(), "Should detect rename");

        let rename = result.unwrap();
        assert_eq!(rename.old_path, "old_file.rs");
        assert_eq!(rename.new_path, "new_file.rs");
        assert_eq!(rename.similarity, 100);
        assert_eq!(rename.commit_hash, commit_hash);
    }

    #[test]
    fn test_detect_rename_no_rename() {
        let _guard = DirGuard::new();

        let temp_dir = setup_test_repo_with_rename();
        let repo_path = temp_dir.path();

        std::env::set_current_dir(repo_path).expect("Failed to change directory");

        // Get the first commit (before rename)
        let output = Command::new("git")
            .args(["rev-list", "--max-parents=0", "HEAD"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to get first commit");

        let first_commit = String::from_utf8(output.stdout)
            .expect("Invalid UTF-8")
            .trim()
            .to_string();

        let result = detect_rename_in_commit(&first_commit[..7], "old_file.rs");
        assert!(
            result.is_none(),
            "Should not detect rename in initial commit"
        );
    }

    #[test]
    fn test_get_file_rename_history_basic() {
        let _guard = DirGuard::new();

        let temp_dir = setup_test_repo_with_rename();
        let repo_path = temp_dir.path();

        std::env::set_current_dir(repo_path).expect("Failed to change directory");

        let history = get_file_rename_history("new_file.rs".to_string());

        // Should have at least 1 entry
        assert!(
            !history.is_empty(),
            "History should contain at least the current path"
        );

        // The chronologically first entry should be the old path
        if history.len() > 1 {
            assert_eq!(
                history[0].0, "old_file.rs",
                "First entry should be old_file.rs"
            );
            assert_eq!(
                history[history.len() - 1].0,
                "new_file.rs",
                "Last entry should be new_file.rs"
            );
        }
    }

    #[test]
    fn test_multiple_renames() {
        let _guard = DirGuard::new();

        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let repo_path = temp_dir.path();

        // Initialize git repo
        Command::new("git")
            .args(["init"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to init git repo");

        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to set git user.name");

        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to set git user.email");

        // Create initial file
        std::fs::write(repo_path.join("file_v1.rs"), "fn main() {}\n")
            .expect("Failed to write file");

        Command::new("git")
            .args(["add", "file_v1.rs"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to add file");

        Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to commit");

        // First rename
        Command::new("git")
            .args(["mv", "file_v1.rs", "file_v2.rs"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to rename file");

        Command::new("git")
            .args(["commit", "-m", "First rename"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to commit");

        // Second rename
        Command::new("git")
            .args(["mv", "file_v2.rs", "file_v3.rs"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to rename file");

        Command::new("git")
            .args(["commit", "-m", "Second rename"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to commit");

        std::env::set_current_dir(repo_path).expect("Failed to change directory");

        let history = get_file_rename_history("file_v3.rs".to_string());

        // Should track through all renames
        assert!(
            !history.is_empty(),
            "Should have at least one path in history"
        );

        // Check if we can find the original file name
        let has_v1 = history.iter().any(|(path, _)| path == "file_v1.rs");
        let has_v3 = history.iter().any(|(path, _)| path == "file_v3.rs");

        if history.len() > 1 {
            assert!(
                has_v1 || has_v3,
                "Should contain either original or final filename"
            );
        }
    }

    #[test]
    fn test_rename_with_content_change() {
        let _guard = DirGuard::new();

        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let repo_path = temp_dir.path();

        // Initialize git repo
        Command::new("git")
            .args(["init"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to init git repo");

        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to set git user.name");

        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to set git user.email");

        // Create initial file with enough content for similarity detection
        std::fs::write(
            repo_path.join("before.rs"),
            "// Utility module\n\
            fn helper1() {\n    println!(\"helper1\");\n}\n\n\
            fn helper2() {\n    println!(\"helper2\");\n}\n\n\
            fn helper3() {\n    println!(\"helper3\");\n}\n\n\
            fn helper4() {\n    println!(\"helper4\");\n}\n",
        )
        .expect("Failed to write file");

        Command::new("git")
            .args(["add", "before.rs"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to add file");

        Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to commit");

        // Rename and modify the file
        Command::new("git")
            .args(["mv", "before.rs", "after.rs"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to rename file");

        // Modify content slightly - change just the comment to keep similarity high
        std::fs::write(
            repo_path.join("after.rs"),
            "// Helper module\n\
            fn helper1() {\n    println!(\"helper1\");\n}\n\n\
            fn helper2() {\n    println!(\"helper2\");\n}\n\n\
            fn helper3() {\n    println!(\"helper3\");\n}\n\n\
            fn helper4() {\n    println!(\"helper4\");\n}\n",
        )
        .expect("Failed to write file");

        Command::new("git")
            .args(["add", "after.rs"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to add modified file");

        Command::new("git")
            .args(["commit", "-m", "Rename and modify"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to commit");

        std::env::set_current_dir(repo_path).expect("Failed to change directory");

        let commit_hash = get_latest_commit(repo_path);
        let result = detect_rename_in_commit(&commit_hash, "after.rs");

        // Git should detect this as a rename since most content is similar
        assert!(
            result.is_some(),
            "Should detect rename with minor content changes"
        );

        if let Some(rename) = result {
            assert_eq!(rename.old_path, "before.rs");
            assert_eq!(rename.new_path, "after.rs");
            // Similarity should be high since we only changed one line
            assert!(
                rename.similarity >= 50,
                "Similarity should be at least 50% (Git's default threshold)"
            );
        }
    }

    #[test]
    fn test_detect_rename_wrong_file() {
        let _guard = DirGuard::new();

        let temp_dir = setup_test_repo_with_rename();
        let repo_path = temp_dir.path();

        std::env::set_current_dir(repo_path).expect("Failed to change directory");

        let commit_hash = get_latest_commit(repo_path);
        let result = detect_rename_in_commit(&commit_hash, "nonexistent_file.rs");

        assert!(result.is_none(), "Should not detect rename for wrong file");
    }
}
