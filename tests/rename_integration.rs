// Integration tests for file rename/move detection with full indexing workflow
use contextpilot::git_command_algo::{get_all_commits_for_file, index_some_commits};
use std::process::Command;
use tempfile::TempDir;

#[cfg(test)]
mod tests_rename_integration {
    use super::*;

    /// Helper to create a comprehensive test repository
    fn setup_comprehensive_test_repo() -> TempDir {
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

        // Commit 1: Create initial file with some content
        std::fs::write(
            repo_path.join("utils.rs"),
            "// Utility functions\n\
pub fn add(a: i32, b: i32) -> i32 {\n\
    a + b\n\
}\n\
\n\
pub fn subtract(a: i32, b: i32) -> i32 {\n\
    a - b\n\
}\n",
        )
        .expect("Failed to write initial file");

        Command::new("git")
            .args(["add", "utils.rs"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to add file");

        Command::new("git")
            .args(["commit", "-m", "Add utility functions"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to commit");

        // Commit 2: Modify the file (add a function)
        std::fs::write(
            repo_path.join("utils.rs"),
            "// Utility functions\n\
pub fn add(a: i32, b: i32) -> i32 {\n\
    a + b\n\
}\n\
\n\
pub fn subtract(a: i32, b: i32) -> i32 {\n\
    a - b\n\
}\n\
\n\
pub fn multiply(a: i32, b: i32) -> i32 {\n\
    a * b\n\
}\n",
        )
        .expect("Failed to write modified file");

        Command::new("git")
            .args(["add", "utils.rs"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to add modified file");

        Command::new("git")
            .args(["commit", "-m", "Add multiply function"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to commit");

        // Commit 3: Rename the file
        Command::new("git")
            .args(["mv", "utils.rs", "math_utils.rs"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to rename file");

        Command::new("git")
            .args(["commit", "-m", "Rename utils.rs to math_utils.rs"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to commit rename");

        // Commit 4: Modify after rename
        std::fs::write(
            repo_path.join("math_utils.rs"),
            "// Math utility functions\n\
pub fn add(a: i32, b: i32) -> i32 {\n\
    a + b\n\
}\n\
\n\
pub fn subtract(a: i32, b: i32) -> i32 {\n\
    a - b\n\
}\n\
\n\
pub fn multiply(a: i32, b: i32) -> i32 {\n\
    a * b\n\
}\n\
\n\
pub fn divide(a: i32, b: i32) -> i32 {\n\
    a / b\n\
}\n",
        )
        .expect("Failed to write file after rename");

        Command::new("git")
            .args(["add", "math_utils.rs"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to add modified file");

        Command::new("git")
            .args(["commit", "-m", "Add divide function"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to commit");

        temp_dir
    }

    #[tokio::test]
    async fn test_get_all_commits_follows_renames() {
        let temp_dir = setup_comprehensive_test_repo();
        let repo_path = temp_dir.path();

        std::env::set_current_dir(repo_path).expect("Failed to change directory");

        // Get all commits for the renamed file
        let commits = get_all_commits_for_file("math_utils.rs".to_string());

        // Should include commits from before the rename
        assert!(
            commits.len() >= 4,
            "Should have at least 4 commits (including pre-rename history)"
        );

        println!("Found commits: {commits:?}");

        // Verify we can trace back to the original file
        // The commits should include those that touched utils.rs before the rename
    }

    #[tokio::test]
    async fn test_index_some_commits_with_rename() {
        let temp_dir = setup_comprehensive_test_repo();
        let repo_path = temp_dir.path();

        std::env::set_current_dir(repo_path).expect("Failed to change directory");

        // Get all commits
        let commits = get_all_commits_for_file("math_utils.rs".to_string());

        println!("Commits to index: {commits:?}");

        // Index all commits
        let auth_details = index_some_commits("math_utils.rs".to_string(), commits).await;

        // Should have indexed lines from the file
        assert!(!auth_details.is_empty(), "Should have indexed some lines");

        println!("Indexed {} lines", auth_details.len());

        // Verify that we have commit history for lines
        for (line_num, details) in auth_details.iter() {
            println!(
                "Line {}: {:?} commits",
                line_num,
                details.commit_hashes.len()
            );
            assert!(
                !details.commit_hashes.is_empty(),
                "Line {line_num} should have at least one commit"
            );
        }
    }

    #[tokio::test]
    async fn test_line_history_across_rename() {
        let temp_dir = setup_comprehensive_test_repo();
        let repo_path = temp_dir.path();

        std::env::set_current_dir(repo_path).expect("Failed to change directory");

        let commits = get_all_commits_for_file("math_utils.rs".to_string());
        let auth_details = index_some_commits("math_utils.rs".to_string(), commits.clone()).await;

        // The `add` function should have commits from before the rename
        // Line 2-4 contains the add function
        let add_function_lines: Vec<_> = auth_details
            .iter()
            .filter(|(line_num, _)| **line_num >= 2 && **line_num <= 4)
            .collect();

        assert!(
            !add_function_lines.is_empty(),
            "Should find add function lines"
        );

        // Check that at least one line has commit history
        let has_history = add_function_lines
            .iter()
            .any(|(_, details)| !details.commit_hashes.is_empty());

        assert!(
            has_history,
            "Add function should have commit history from before rename"
        );
    }

    #[tokio::test]
    async fn test_accuracy_with_git_blame_across_rename() {
        let temp_dir = setup_comprehensive_test_repo();
        let repo_path = temp_dir.path();

        std::env::set_current_dir(repo_path).expect("Failed to change directory");

        let commits = get_all_commits_for_file("math_utils.rs".to_string());
        let auth_details = index_some_commits("math_utils.rs".to_string(), commits.clone()).await;

        // Get the latest commit for blame comparison
        let latest_commit = commits.last().expect("Should have commits");

        let mut total_count = 0;
        let mut match_count = 0;

        for (line_num, details) in auth_details.iter() {
            // Get git blame for this line
            let output = Command::new("git")
                .args([
                    "blame",
                    latest_commit,
                    "-L",
                    &format!("{line_num},{line_num}"),
                    "--abbrev=7",
                    "--",
                    "math_utils.rs",
                ])
                .current_dir(repo_path)
                .output()
                .expect("Failed to run git blame");

            if !output.status.success() {
                continue;
            }

            let stdout = String::from_utf8(output.stdout).unwrap_or_default();
            let first_line = match stdout.lines().next() {
                Some(line) => line,
                None => continue,
            };

            let parts: Vec<&str> = first_line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            let mut blame_commit = parts[0].to_string();

            // Handle the ^ prefix for boundary commits
            if blame_commit.starts_with('^') {
                blame_commit = blame_commit.trim_start_matches('^').to_string();
            }

            // Ensure we're comparing 7-character hashes
            if blame_commit.len() > 7 {
                blame_commit = blame_commit[..7].to_string();
            }

            total_count += 1;

            // Check if our indexed commits contain the blame commit
            if details.commit_hashes.contains(&blame_commit) {
                match_count += 1;
            } else {
                println!(
                    "Line {}: Expected commit {} not found in {:?}",
                    line_num, blame_commit, details.commit_hashes
                );
            }
        }

        println!(
            "Accuracy: {}/{} ({:.2}%)",
            match_count,
            total_count,
            (match_count as f64 / total_count as f64) * 100.0
        );

        // We should have at least 80% accuracy
        let accuracy = (match_count as f64 / total_count as f64) * 100.0;
        assert!(
            accuracy >= 80.0,
            "Accuracy should be at least 80%, got {accuracy:.2}%"
        );
    }

    #[tokio::test]
    async fn test_multiple_renames_full_workflow() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let repo_path = temp_dir.path();

        // Initialize repo
        Command::new("git")
            .args(["init"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to init");

        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to set user.name");

        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to set user.email");

        // Create and commit initial file
        std::fs::write(
            repo_path.join("v1.rs"),
            "pub fn hello() {\n    println!(\"v1\");\n}\n",
        )
        .expect("Failed to write");

        Command::new("git")
            .args(["add", "v1.rs"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to add");

        Command::new("git")
            .args(["commit", "-m", "v1"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to commit");

        // First rename
        Command::new("git")
            .args(["mv", "v1.rs", "v2.rs"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to rename");

        Command::new("git")
            .args(["commit", "-m", "rename to v2"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to commit");

        // Second rename
        Command::new("git")
            .args(["mv", "v2.rs", "v3.rs"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to rename");

        Command::new("git")
            .args(["commit", "-m", "rename to v3"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to commit");

        std::env::set_current_dir(repo_path).expect("Failed to change directory");

        let commits = get_all_commits_for_file("v3.rs".to_string());

        // Should have 3 commits
        assert_eq!(commits.len(), 3, "Should have 3 commits across all renames");

        let auth_details = index_some_commits("v3.rs".to_string(), commits).await;

        // Should be able to index the file
        assert!(
            !auth_details.is_empty(),
            "Should successfully index renamed file"
        );
    }
}
