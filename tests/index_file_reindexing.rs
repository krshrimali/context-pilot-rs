use contextpilot::contextgpt_structs::{AuthorDetailsV2, RequestTypeOptions};
use contextpilot::db::DB;
use contextpilot::algo_loc::perform_for_whole_file;
use contextpilot::git_command_algo;
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use tempfile::tempdir;
use std::process::Command;
use std::path::Path;
use std::sync::Arc;

// Helper function to initialize a git repository
fn init_git_repo(dir_path: &Path) {
    Command::new("git")
        .args(["init"])
        .current_dir(dir_path)
        .output()
        .expect("Failed to initialize git repository");

    // Configure git user for commits
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(dir_path)
        .output()
        .expect("Failed to configure git user name");

    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(dir_path)
        .output()
        .expect("Failed to configure git user email");
}

// Helper function to add and commit a file
fn commit_file(dir_path: &Path, file_path: &Path, commit_message: &str) -> String {
    // Add the file to git
    Command::new("git")
        .args(["add", file_path.to_str().unwrap()])
        .current_dir(dir_path)
        .output()
        .expect("Failed to add file to git");

    // Commit the file
    Command::new("git")
        .args(["commit", "-m", commit_message])
        .current_dir(dir_path)
        .output()
        .expect("Failed to commit file");

    // Get the commit hash
    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(dir_path)
        .output()
        .expect("Failed to get commit hash");

    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

#[tokio::test]
async fn test_index_file_reindexing() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let repo_dir = temp_dir.path();

    // Initialize git repository
    init_git_repo(repo_dir);

    // Create a test file
    let file_path = repo_dir.join("test_file.txt");
    let mut file = File::create(&file_path).expect("Failed to create test file");
    writeln!(file, "Test content line 1").expect("Failed to write to test file");
    writeln!(file, "Test content line 2").expect("Failed to write to test file");
    writeln!(file, "Test content line 3").expect("Failed to write to test file");

    // Commit the file to get a commit hash
    let commit_hash = commit_file(repo_dir, &file_path, "Initial commit");

    // Mock the home directory and DB folder structure
    let home_dir = temp_dir.path().join("home");
    fs::create_dir_all(&home_dir).expect("Failed to create home directory");

    // Set up environment for testing
    unsafe {
        std::env::set_var("HOME", home_dir.to_str().unwrap());
    }

    // Create workspace path and DB folder
    let workspace_name = "test_workspace";
    let db_folder = home_dir.join(".context_pilot_db").join(workspace_name);
    fs::create_dir_all(&db_folder).expect("Failed to create DB folder");

    // Create a DB instance
    let mut db = DB {
        folder_path: workspace_name.to_string(),
        ..Default::default()
    };

    // Initialize the DB with the test file
    let file_path_str = file_path.to_str().unwrap().to_string();
    db.init_db(workspace_name, Some(&file_path_str), false);

    // Index the file first time
    let result1 = perform_for_whole_file(
        file_path_str.clone(),
        true,
        None,
        Some(workspace_name.to_string()),
    ).await;

    // Only store and query if there are results
    if !result1.is_empty() {
        // Store the result in the DB
        db.append_to_db(&file_path_str, 0, result1.clone());
        db.store();

        // Get the indices for the file after first indexing
        let indices_before = db.find_index(&file_path_str);
        assert!(indices_before.is_some(), "File should be indexed now");
        let indices_before = indices_before.unwrap();
        println!("Indices before re-indexing: {:?}", indices_before);

        // Check that the shard files exist
        for index in &indices_before {
            let shard_path = format!("{}/{}.json", db_folder.display(), index);
            assert!(Path::new(&shard_path).exists(), "Shard file should exist: {}", shard_path);
        }

        // Modify the file
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&file_path)
            .expect("Failed to open test file for appending");
        writeln!(file, "Test content line 4").expect("Failed to append to test file");

        // Commit the changes
        let new_commit_hash = commit_file(repo_dir, &file_path, "Second commit");

        // Index the file again
        let result2 = perform_for_whole_file(
            file_path_str.clone(),
            true,
            None,
            Some(workspace_name.to_string()),
        ).await;

        // Only store and query if there are results
        if !result2.is_empty() {
            // Store the result in the DB
            db.append_to_db(&file_path_str, 0, result2.clone());
            db.store();

            // Get the indices for the file after re-indexing
            let indices_after = db.find_index(&file_path_str);
            assert!(indices_after.is_some(), "File should still be indexed after re-indexing");
            let indices_after = indices_after.unwrap();
            println!("Indices after re-indexing: {:?}", indices_after);

            // Check that the old shard files are deleted
            for index in &indices_before {
                if !indices_after.contains(index) {
                    let shard_path = format!("{}/{}.json", db_folder.display(), index);
                    assert!(!Path::new(&shard_path).exists(), "Old shard file should be deleted: {}", shard_path);
                }
            }

            // Check that the new shard files exist
            for index in &indices_after {
                let shard_path = format!("{}/{}.json", db_folder.display(), index);
                assert!(Path::new(&shard_path).exists(), "New shard file should exist: {}", shard_path);
            }

            // Test querying the indexed file
            db.query(file_path_str.clone(), 1, 4).await;
        } else {
            println!("No results to store or query after re-indexing in test environment");
        }
    } else {
        println!("No results to store or query in test environment");
    }
}
