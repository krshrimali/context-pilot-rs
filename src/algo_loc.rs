use crate::git_command_algo::get_latest_commit;
use crate::git_command_algo::index_some_commits;
use crate::{
    config, contextgpt_structs::AuthorDetailsV2, git_command_algo::extract_details_parallel,
};
use std::collections::HashMap;
use std::path::PathBuf;

pub async fn perform_for_whole_file(
    origin_file_path: String,
    should_print: bool,
    commits_to_index: Option<Vec<String>>,
    workspace_path: Option<String>,
) -> HashMap<u32, AuthorDetailsV2> {
    if let Some(workspace) = workspace_path.as_ref() {
        match get_latest_commit(&origin_file_path) {
            Some(recent_commit) => {
                if let Some(home) = simple_home_dir::home_dir() {
                    let workspace_path_buf = PathBuf::from(workspace);
                    let workspace_name = workspace_path_buf
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("default_workspace");

                    println!(
                        "DEBUG: Workspace name: {}",
                        workspace_path_buf.to_string_lossy()
                    );
                    let db_folder = format!(
                        "{}{}{}{}{}",
                        home.to_string_lossy(),
                        std::path::MAIN_SEPARATOR,
                        config::DB_FOLDER,
                        std::path::MAIN_SEPARATOR,
                        workspace_path_buf.to_string_lossy().to_string()
                    );

                    if let path_str = db_folder.to_string() {
                        // let indexing_path = db_folder.join(path_str).join("indexing_metadata.json");
                        let indexing_path = format!(
                            "{}{}indexing_metadata.json",
                            path_str,
                            std::path::MAIN_SEPARATOR
                        );

                        if PathBuf::from(indexing_path.clone()).exists() {
                            match std::fs::read_to_string(&indexing_path) {
                                Ok(metadata_str) => {
                                    match serde_json::from_str::<HashMap<String, Vec<String>>>(
                                        &metadata_str,
                                    ) {
                                        Ok(indexing_metadata) => {
                                            // Normalize paths for proper comparison
                                            let origin_path_buf = PathBuf::from(&origin_file_path);
                                            let workspace_path_buf = PathBuf::from(workspace);

                                            // Try to canonicalize both paths for accurate comparison
                                            let canonical_origin = origin_path_buf
                                                .canonicalize()
                                                .unwrap_or(origin_path_buf.clone());
                                            let canonical_workspace = workspace_path_buf
                                                .canonicalize()
                                                .unwrap_or(workspace_path_buf.clone());

                                            let relative_path = if let Ok(rel) =
                                                canonical_origin.strip_prefix(&canonical_workspace)
                                            {
                                                rel.to_string_lossy().replace('\\', "/")
                                            } else {
                                                // Fallback: try without canonicalization
                                                if let Ok(rel) = origin_path_buf
                                                    .strip_prefix(&workspace_path_buf)
                                                {
                                                    rel.to_string_lossy().replace('\\', "/")
                                                } else {
                                                    // Last resort: use filename
                                                    origin_path_buf
                                                        .file_name()
                                                        .map(|name| {
                                                            name.to_string_lossy().to_string()
                                                        })
                                                        .unwrap_or_else(|| origin_file_path.clone())
                                                }
                                            };

                                            // Check if file is already indexed with latest commit
                                            // Try different path variations to find existing metadata
                                            let possible_paths = vec![
                                                origin_file_path.clone(),
                                                if let Ok(canonical) =
                                                    PathBuf::from(&origin_file_path).canonicalize()
                                                {
                                                    canonical.to_string_lossy().to_string()
                                                } else {
                                                    origin_file_path.clone()
                                                },
                                                relative_path.clone(),
                                            ];

                                            println!(
                                                "DEBUG: Checking if file is already indexed with latest commit: {}",
                                                recent_commit
                                            );
                                            println!("DEBUG: Possible paths to check:");
                                            for (i, path) in possible_paths.iter().enumerate() {
                                                println!("DEBUG:   {}: {}", i, path);
                                            }

                                            for path_variant in possible_paths {
                                                println!(
                                                    "DEBUG: Checking path variant: {}",
                                                    path_variant
                                                );
                                                if let Some(last_indexing_data) =
                                                    indexing_metadata.get(&path_variant)
                                                {
                                                    println!(
                                                        "DEBUG: Found indexing data for path: {}",
                                                        path_variant
                                                    );
                                                    println!(
                                                        "DEBUG: Last indexing data: {:?}",
                                                        last_indexing_data
                                                    );

                                                    if let Some(last_indexed_commit) =
                                                        last_indexing_data.last()
                                                    {
                                                        println!(
                                                            "DEBUG: Last indexed commit: {}",
                                                            last_indexed_commit
                                                        );

                                                        if last_indexed_commit == &recent_commit {
                                                            println!(
                                                                "DEBUG: Commit matches! No reindexing needed."
                                                            );
                                                            if should_print {
                                                                println!(
                                                                    "File {} is already indexed with the latest commit {}",
                                                                    path_variant, recent_commit
                                                                );
                                                            }
                                                            return HashMap::new();
                                                        } else {
                                                            println!(
                                                                "DEBUG: Commit doesn't match. Reindexing needed."
                                                            );
                                                            if should_print {
                                                                println!(
                                                                    "File {} has a new commit {} (last indexed: {})",
                                                                    path_variant,
                                                                    recent_commit,
                                                                    last_indexed_commit
                                                                );
                                                            }
                                                        }
                                                    } else {
                                                        println!(
                                                            "DEBUG: No last indexed commit found."
                                                        );
                                                    }
                                                } else {
                                                    println!(
                                                        "DEBUG: No indexing data found for path: {}",
                                                        path_variant
                                                    );
                                                }
                                            }

                                            println!(
                                                "DEBUG: No matching indexed commit found. Proceeding with indexing."
                                            );
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
