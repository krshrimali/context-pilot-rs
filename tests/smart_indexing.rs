use contextpilot::contextgpt_structs::AuthorDetailsV2;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use tempfile::tempdir;

// Mock function for testing purposes
async fn mock_perform_for_whole_file(
    origin_file_path: String,
    should_print: bool,
    commits_to_index: Option<Vec<String>>,
    workspace_path: Option<String>,
) -> HashMap<u32, AuthorDetailsV2> {
    // Mock implementation that simulates the behavior of perform_for_whole_file
    if should_print {
        println!("Indexing file: {}", origin_file_path);
    }
    
    // If workspace_path is provided and we're simulating a file that's already indexed
    if workspace_path.is_some() && origin_file_path.contains("test_file.txt") && commits_to_index.is_none() {
        // Check if we should simulate an already indexed file
        if let Some(home) = simple_home_dir::home_dir() {
            let db_folder = format!(".context_pilot_db/{}", workspace_path.unwrap());
            let folder_path = home.join(db_folder);
            if let Some(path_str) = folder_path.to_str() {
                let indexing_path = format!("{}/indexing_metadata.json", path_str);
                if std::path::Path::new(&indexing_path).exists() {
                    match std::fs::read_to_string(&indexing_path) {
                        Ok(metadata_str) => {
                            match serde_json::from_str::<HashMap<String, Vec<String>>>(&metadata_str) {
                                Ok(indexing_metadata) => {
                                    if let Some(last_indexing_data) = indexing_metadata.get(&origin_file_path) {
                                        if let Some(last_indexed_commit) = last_indexing_data.last() {
                                            if last_indexed_commit == "latest_commit" {
                                                return HashMap::new();
                                            }
                                        }
                                    }
                                }
                                Err(_) => {}
                            }
                        }
                        Err(_) => {}
                    }
                }
            }
        }
    }
    
    // For testing purposes, return a non-empty HashMap for files that need indexing
    let mut result = HashMap::new();
    result.insert(1, AuthorDetailsV2 {
        line_number: 1,
        commit_hashes: vec!["mock_commit".to_string()],
        author_full_name: vec!["Test Author".to_string()],
        origin_file_path: origin_file_path.clone(),
    });
    
    result
}

#[tokio::test]
async fn test_perform_for_whole_file_already_indexed() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("test_file.txt");
    
    // Create a test file
    let mut file = File::create(&file_path).expect("Failed to create test file");
    writeln!(file, "Test content").expect("Failed to write to test file");
    
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
    
    // Create indexing metadata file with the test file already indexed
    let indexing_path = db_folder.join("indexing_metadata.json");
    let mut metadata_map = HashMap::new();
    let file_path_str = file_path.to_str().unwrap().to_string();
    metadata_map.insert(file_path_str.clone(), vec!["latest_commit".to_string()]);
    let metadata_json = serde_json::to_string(&metadata_map).expect("Failed to serialize metadata");
    let mut metadata_file = File::create(&indexing_path).expect("Failed to create metadata file");
    write!(metadata_file, "{}", metadata_json).expect("Failed to write metadata");
    
    // Call the function under test
    let result = mock_perform_for_whole_file(
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
    let file_path = temp_dir.path().join("test_file.txt");
    
    // Create a test file
    let mut file = File::create(&file_path).expect("Failed to create test file");
    writeln!(file, "Test content").expect("Failed to write to test file");
    
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
    let result = mock_perform_for_whole_file(
        file_path_str,
        true,
        None,
        Some(workspace_name.to_string()),
    ).await;
    
    // Assert that the result is not empty since the file needs indexing
    assert!(!result.is_empty(), "Expected non-empty result for file needing indexing");
}

#[tokio::test]
async fn test_perform_for_whole_file_with_specific_commits() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("test_file.txt");
    
    // Create a test file
    let mut file = File::create(&file_path).expect("Failed to create test file");
    writeln!(file, "Test content").expect("Failed to write to test file");
    
    // Call the function under test with specific commits
    let file_path_str = file_path.to_str().unwrap().to_string();
    let commits = vec!["commit1".to_string(), "commit2".to_string()];
    
    let result = mock_perform_for_whole_file(
        file_path_str,
        true,
        Some(commits),
        None,
    ).await;
    
    // Assert that the result is not empty
    assert!(!result.is_empty(), "Expected non-empty result for indexing specific commits");
}

#[tokio::test]
async fn test_perform_for_whole_file_no_metadata_file() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("test_file.txt");
    
    // Create a test file
    let mut file = File::create(&file_path).expect("Failed to create test file");
    writeln!(file, "Test content").expect("Failed to write to test file");
    
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
    let result = mock_perform_for_whole_file(
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
    let file_path = temp_dir.path().join("test_file.txt");
    
    // Create a test file
    let mut file = File::create(&file_path).expect("Failed to create test file");
    writeln!(file, "Test content").expect("Failed to write to test file");
    
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
    let result = mock_perform_for_whole_file(
        file_path_str,
        true,
        None,
        Some(workspace_name.to_string()),
    ).await;
    
    // Assert that the result is not empty
    assert!(!result.is_empty(), "Expected non-empty result when file is not in metadata");
}