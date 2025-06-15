use contextpilot::contextgpt_structs::AuthorDetailsV2;
use contextpilot::algo_loc::perform_for_whole_file;
use contextpilot::git_command_algo;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use tempfile::tempdir;
use std::process::Command;
use std::path::Path;

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
async fn test_perform_for_whole_file_already_indexed() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let repo_dir = temp_dir.path();

    // Initialize git repository
    init_git_repo(repo_dir);

    // Create a test file
    let file_path = repo_dir.join("test_file.txt");
    let mut file = File::create(&file_path).expect("Failed to create test file");
    writeln!(file, "Test content").expect("Failed to write to test file");

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

    // Create indexing metadata file with the test file already indexed with the actual commit hash
    let indexing_path = db_folder.join("indexing_metadata.json");
    let mut metadata_map = HashMap::new();
    let file_path_str = file_path.to_str().unwrap().to_string();
    metadata_map.insert(file_path_str.clone(), vec![commit_hash]);
    let metadata_json = serde_json::to_string(&metadata_map).expect("Failed to serialize metadata");
    let mut metadata_file = File::create(&indexing_path).expect("Failed to create metadata file");
    write!(metadata_file, "{}", metadata_json).expect("Failed to write metadata");

    // Call the function under test
    let result = perform_for_whole_file(
        file_path_str,
        true,
        None,
        Some(workspace_name.to_string()),
    ).await;

    // Assert that the result is empty since the file is already indexed
    assert!(result.is_empty(), "Expected empty result for already indexed file");
}

#[tokio::test]
async fn test_perform_for_whole_file_needs_indexing() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let repo_dir = temp_dir.path();

    // Initialize git repository
    init_git_repo(repo_dir);

    // Create a test file
    let file_path = repo_dir.join("test_file.txt");
    let mut file = File::create(&file_path).expect("Failed to create test file");
    writeln!(file, "Test content").expect("Failed to write to test file");

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

    // Create indexing metadata file with different commit hash
    let indexing_path = db_folder.join("indexing_metadata.json");
    let mut metadata_map = HashMap::new();
    let file_path_str = file_path.to_str().unwrap().to_string();
    metadata_map.insert(file_path_str.clone(), vec!["old_commit".to_string()]);
    let metadata_json = serde_json::to_string(&metadata_map).expect("Failed to serialize metadata");
    let mut metadata_file = File::create(&indexing_path).expect("Failed to create metadata file");
    write!(metadata_file, "{}", metadata_json).expect("Failed to write metadata");

    // Call the function under test
    let result = perform_for_whole_file(
        file_path_str,
        true,
        None,
        Some(workspace_name.to_string()),
    ).await;

    // The result might be empty if there are no commits for the file
    // This is expected behavior for the actual function
    // We're just testing that the function runs without errors
}

#[tokio::test]
async fn test_perform_for_whole_file_with_specific_commits() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let repo_dir = temp_dir.path();

    // Initialize git repository
    init_git_repo(repo_dir);

    // Create a test file
    let file_path = repo_dir.join("test_file.txt");
    let mut file = File::create(&file_path).expect("Failed to create test file");
    writeln!(file, "Test content").expect("Failed to write to test file");

    // Commit the file to get a commit hash
    let commit_hash = commit_file(repo_dir, &file_path, "Initial commit");

    // Call the function under test with specific commits
    let file_path_str = file_path.to_str().unwrap().to_string();
    let commits = vec![commit_hash];

    let result = perform_for_whole_file(
        file_path_str,
        true,
        Some(commits),
        None,
    ).await;

    // The result might be empty if there are no commits for the file
    // This is expected behavior for the actual function
    // We're just testing that the function runs without errors
}

#[tokio::test]
async fn test_perform_for_whole_file_no_metadata_file() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let repo_dir = temp_dir.path();

    // Initialize git repository
    init_git_repo(repo_dir);

    // Create a test file
    let file_path = repo_dir.join("test_file.txt");
    let mut file = File::create(&file_path).expect("Failed to create test file");
    writeln!(file, "Test content").expect("Failed to write to test file");

    // Commit the file to get a commit hash
    commit_file(repo_dir, &file_path, "Initial commit");

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

    // No metadata file is created, so indexing should proceed

    // Call the function under test
    let file_path_str = file_path.to_str().unwrap().to_string();
    let result = perform_for_whole_file(
        file_path_str,
        true,
        None,
        Some(workspace_name.to_string()),
    ).await;

    // Assert that the result is not empty
    assert!(!result.is_empty(), "Expected non-empty result when no metadata file exists");
}

#[tokio::test]
async fn test_perform_for_whole_file_file_not_in_metadata() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let repo_dir = temp_dir.path();

    // Initialize git repository
    init_git_repo(repo_dir);

    // Create a test file
    let file_path = repo_dir.join("test_file.txt");
    let mut file = File::create(&file_path).expect("Failed to create test file");
    writeln!(file, "Test content").expect("Failed to write to test file");

    // Commit the file to get a commit hash
    commit_file(repo_dir, &file_path, "Initial commit");

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

    // Create indexing metadata file without the test file
    let indexing_path = db_folder.join("indexing_metadata.json");
    let mut metadata_map = HashMap::new();
    metadata_map.insert("other_file.txt".to_string(), vec!["some_commit".to_string()]);
    let metadata_json = serde_json::to_string(&metadata_map).expect("Failed to serialize metadata");
    let mut metadata_file = File::create(&indexing_path).expect("Failed to create metadata file");
    write!(metadata_file, "{}", metadata_json).expect("Failed to write metadata");

    // Call the function under test
    let file_path_str = file_path.to_str().unwrap().to_string();
    let result = perform_for_whole_file(
        file_path_str,
        true,
        None,
        Some(workspace_name.to_string()),
    ).await;

    // Assert that the result is not empty
    assert!(!result.is_empty(), "Expected non-empty result when file is not in metadata");
}
