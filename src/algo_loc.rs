use crate::git_command_algo::get_latest_commit;
use crate::git_command_algo::index_some_commits;
use crate::{
    config, contextgpt_structs::AuthorDetailsV2, git_command_algo::extract_details_parallel,
};
use std::collections::HashMap;

pub async fn perform_for_whole_file(
    origin_file_path: String,
    should_print: bool,
    commits_to_index: Option<Vec<String>>,
    workspace_path: Option<String>,
) -> HashMap<u32, AuthorDetailsV2> {
    // First check if we need to perform indexing at all
    if workspace_path.is_some() {
        match get_latest_commit(&origin_file_path) {
            Some(recent_commit) => {
                // Check if the file exists in indexing metadata
                let db_folder =
                    format!("{}/{}", config::DB_FOLDER, workspace_path.unwrap().clone());
                if let Some(home) = simple_home_dir::home_dir() {
                    let folder_path = home.join(db_folder);
                    if let Some(path_str) = folder_path.to_str() {
                        let indexing_path = format!("{}/indexing_metadata.json", path_str);
                        if std::path::Path::new(&indexing_path).exists() {
                            match std::fs::read_to_string(&indexing_path) {
                                Ok(metadata_str) => {
                                    match serde_json::from_str::<HashMap<String, Vec<String>>>(
                                        &metadata_str,
                                    ) {
                                        Ok(indexing_metadata) => {
                                            if let Some(last_indexing_data) =
                                                indexing_metadata.get(&origin_file_path)
                                            {
                                                if let Some(last_indexed_commit) =
                                                    last_indexing_data.last()
                                                {
                                                    if last_indexed_commit == &recent_commit {
                                                        if should_print {
                                                            println!(
                                                                "File {} is already indexed with the latest commit {}",
                                                                origin_file_path, recent_commit
                                                            );
                                                        }
                                                        return HashMap::new();
                                                    }
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to parse metadata JSON check: {}", e)
                                        }
                                    }
                                }
                                Err(e) => eprintln!("Failed to read metadata file: {}", e),
                            }
                        }
                    }
                }
            }
            None => {}
        }
    }

    // Proceed with indexing if needed
    let output: HashMap<u32, AuthorDetailsV2>;
    if should_print {
        println!("Indexing file: {}", origin_file_path);
    }
    if commits_to_index.is_none() {
        output = extract_details_parallel(origin_file_path.clone()).await;
    } else {
        output = index_some_commits(origin_file_path.clone(), commits_to_index.unwrap()).await;
    }
    output
}
