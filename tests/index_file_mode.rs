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
async fn test_index_file_mode() {
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
    unsafe { std::env::set_var("HOME", home_dir.to_str().unwrap()); }

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

    // Test that the file is not yet indexed
    let indices = db.find_index(&file_path_str);
    assert!(indices.is_none(), "File should not be indexed yet");

    // Index the file using perform_for_whole_file
    let result = perform_for_whole_file(
        file_path_str.clone(),
        true,
        None,
        Some(workspace_name.to_string()),
    ).await;

    // Verify that the result contains data
    assert!(!result.is_empty(), "Expected non-empty result after indexing");

    // Store the result in the DB
    db.append_to_db(&file_path_str, 0, result.clone());
    db.store();

    // Test that the file is now indexed
    let indices = db.find_index(&file_path_str);
    assert!(indices.is_some(), "File should be indexed now");

    // Test querying the indexed file
    db.query(file_path_str.clone(), 1, 3).await;

    // Test querying descriptions for the indexed file
    db.query_descriptions(file_path_str.clone(), 1, 3).await;
}

#[tokio::test]
async fn test_index_file_mode_with_existing_index() {
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
    unsafe { std::env::set_var("HOME", home_dir.to_str().unwrap()); }

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

    // Store the result in the DB
    db.append_to_db(&file_path_str, 0, result1.clone());
    db.store();

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

    // Verify that the result contains data
    assert!(!result2.is_empty(), "Expected non-empty result after re-indexing");

    // Store the result in the DB
    db.append_to_db(&file_path_str, 0, result2.clone());
    db.store();

    // Test querying the indexed file
    db.query(file_path_str.clone(), 1, 4).await;
}

#[tokio::test]
async fn test_index_file_mode_with_specific_commits() {
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

    // Commit the file to get a commit hash
    let commit_hash1 = commit_file(repo_dir, &file_path, "Initial commit");

    // Modify the file
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(&file_path)
        .expect("Failed to open test file for appending");
    writeln!(file, "Test content line 3").expect("Failed to append to test file");

    // Commit the changes
    let commit_hash2 = commit_file(repo_dir, &file_path, "Second commit");

    // Mock the home directory and DB folder structure
    let home_dir = temp_dir.path().join("home");
    fs::create_dir_all(&home_dir).expect("Failed to create home directory");

    // Set up environment for testing
    unsafe { std::env::set_var("HOME", home_dir.to_str().unwrap()); }

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

    // Index the file with specific commits
    let result = perform_for_whole_file(
        file_path_str.clone(),
        true,
        Some(vec![commit_hash1.clone(), commit_hash2.clone()]),
        Some(workspace_name.to_string()),
    ).await;

    // Verify that the result contains data
    assert!(!result.is_empty(), "Expected non-empty result after indexing with specific commits");

    // Store the result in the DB
    db.append_to_db(&file_path_str, 0, result.clone());
    db.store();

    // Test querying the indexed file
    db.query(file_path_str.clone(), 1, 3).await;
}

#[tokio::test]
async fn test_index_file_mode_does_not_affect_workspace_indexing() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let repo_dir = temp_dir.path();

    // Initialize git repository
    init_git_repo(repo_dir);

    // Create multiple test files
    let file_path1 = repo_dir.join("test_file1.txt");
    let mut file1 = File::create(&file_path1).expect("Failed to create test file 1");
    writeln!(file1, "Test content file 1").expect("Failed to write to test file 1");

    let file_path2 = repo_dir.join("test_file2.txt");
    let mut file2 = File::create(&file_path2).expect("Failed to create test file 2");
    writeln!(file2, "Test content file 2").expect("Failed to write to test file 2");

    // Commit the files
    let commit_hash1 = commit_file(repo_dir, &file_path1, "Commit file 1");
    let commit_hash2 = commit_file(repo_dir, &file_path2, "Commit file 2");

    // Mock the home directory and DB folder structure
    let home_dir = temp_dir.path().join("home");
    fs::create_dir_all(&home_dir).expect("Failed to create home directory");

    // Set up environment for testing
    unsafe { std::env::set_var("HOME", home_dir.to_str().unwrap()); }

    // Create workspace path and DB folder
    let workspace_name = "test_workspace";
    let db_folder = home_dir.join(".context_pilot_db").join(workspace_name);
    fs::create_dir_all(&db_folder).expect("Failed to create DB folder");

    // Create a DB instance for file 1
    let mut db1 = DB {
        folder_path: workspace_name.to_string(),
        ..Default::default()
    };

    // Initialize the DB with file 1
    let file_path_str1 = file_path1.to_str().unwrap().to_string();
    db1.init_db(workspace_name, Some(&file_path_str1), false);

    // Index file 1
    let result1 = perform_for_whole_file(
        file_path_str1.clone(),
        true,
        None,
        Some(workspace_name.to_string()),
    ).await;

    // Store the result in the DB
    db1.append_to_db(&file_path_str1, 0, result1.clone());
    db1.store();

    // Create a new DB instance for file 2
    let mut db2 = DB {
        folder_path: workspace_name.to_string(),
        ..Default::default()
    };

    // Initialize the DB with file 2
    let file_path_str2 = file_path2.to_str().unwrap().to_string();
    db2.init_db(workspace_name, Some(&file_path_str2), false);

    // Index file 2
    let result2 = perform_for_whole_file(
        file_path_str2.clone(),
        true,
        None,
        Some(workspace_name.to_string()),
    ).await;

    // Store the result in the DB
    db2.append_to_db(&file_path_str2, 0, result2.clone());
    db2.store();

    // Create a new DB instance to check both files
    let mut db_check = DB {
        folder_path: workspace_name.to_string(),
        ..Default::default()
    };

    // Initialize the DB without specifying a file
    db_check.init_db(workspace_name, None, false);

    // Test that both files are indexed
    let indices1 = db_check.find_index(&file_path_str1);
    let indices2 = db_check.find_index(&file_path_str2);

    assert!(indices1.is_some(), "File 1 should be indexed");
    assert!(indices2.is_some(), "File 2 should be indexed");

    // Test querying both files
    db_check.query(file_path_str1.clone(), 1, 1).await;
    db_check.query(file_path_str2.clone(), 1, 1).await;
}
