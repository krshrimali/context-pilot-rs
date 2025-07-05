use crate::git_command_algo::{extract_details_parallel, get_latest_commit, index_some_commits};
use crate::{config, contextgpt_structs::AuthorDetailsV2};
use std::collections::HashMap;
use std::path::Path;

pub async fn perform_for_whole_file(
    origin_file_path: String,
    should_print: bool,
    commits_to_index: Option<Vec<String>>,
    workspace_path: Option<String>,
) -> HashMap<u32, AuthorDetailsV2> {
    // Check if we should skip indexing based on existing metadata
    if let Some(workspace_path) = &workspace_path {
        if is_already_indexed(&origin_file_path, workspace_path, should_print) {
            return HashMap::new();
        }
    }

    // Perform the actual indexing
    index_file(&origin_file_path, commits_to_index, should_print).await
}

fn is_already_indexed(origin_file_path: &str, workspace_path: &str, should_print: bool) -> bool {
    // Get the latest commit safely
    let Some(recent_commit) = get_latest_commit(&origin_file_path.to_string()) else {
        if should_print {
            println!("No commits found for file: {}", origin_file_path);
        }
        return false;
    };

    if should_print {
        println!(
            "Latest commit for file: {} is: {}",
            origin_file_path, recent_commit
        );
    }

    let Some(indexing_path) = build_indexing_path(workspace_path) else {
        return false;
    };

    if should_print {
        println!("Indexing path: {}", indexing_path);
    }

    if !Path::new(&indexing_path).exists() {
        return false;
    }

    match read_indexing_metadata(&indexing_path) {
        Ok(metadata) => {
            if let Some(last_indexed_commit) = get_last_indexed_commit(&metadata, origin_file_path) {
                if should_print {
                    println!("Last indexed commit: {}", last_indexed_commit);
                }
                if last_indexed_commit == recent_commit {
                    if should_print {
                        println!(
                            "File {} is already indexed with the latest commit {}",
                            origin_file_path, recent_commit
                        );
                    }
                    return true;
                }
            }
        }
        Err(e) => {
            if should_print {
                eprintln!("Failed to read or parse indexing metadata: {}", e);
            }
        }
    }

    false
}

fn build_indexing_path(workspace_path: &str) -> Option<String> {
    let processed_workspace_path = if cfg!(target_os = "windows") {
        // Handle Windows paths more safely
        if let Some(stripped) = workspace_path.strip_prefix(r"C:\") {
            stripped
        } else {
            workspace_path
        }
    } else {
        workspace_path
    };

    let db_folder = format!(
        "{}{}{}",
        config::DB_FOLDER,
        std::path::MAIN_SEPARATOR,
        processed_workspace_path
    );
    let home = simple_home_dir::home_dir()?;
    let folder_path = home.join(db_folder);
    let path_str = folder_path.to_str()?;
    Some(format!("{}/indexing_metadata.json", path_str))
}

fn read_indexing_metadata(
    indexing_path: &str,
) -> Result<HashMap<String, Vec<String>>, Box<dyn std::error::Error>> {
    let metadata_str = std::fs::read_to_string(indexing_path)?;
    if metadata_str.trim().is_empty() {
        // Return empty HashMap if file is empty
        return Ok(HashMap::new());
    }
    let metadata = serde_json::from_str(&metadata_str)?;
    Ok(metadata)
}

fn get_last_indexed_commit(
    metadata: &HashMap<String, Vec<String>>,
    origin_file_path: &str,
) -> Option<String> {
    metadata
        .get(origin_file_path)
        .and_then(|commits| commits.last())
        .cloned()
}

async fn index_file(
    origin_file_path: &str,
    commits_to_index: Option<Vec<String>>,
    should_print: bool,
) -> HashMap<u32, AuthorDetailsV2> {
    if should_print {
        println!("Indexing file: {}", origin_file_path);
    }

    match commits_to_index {
        Some(commits) => index_some_commits(origin_file_path.to_string(), commits).await,
        None => extract_details_parallel(origin_file_path.to_string()).await,
    }
}