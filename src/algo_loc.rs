use std::collections::HashMap;
use crate::{git_command_algo::index_some_commits};
use crate::{contextgpt_structs::AuthorDetailsV2, git_command_algo::extract_details_parallel};

pub async fn perform_for_whole_file(
    origin_file_path: String,
    should_print: bool,
    commits_to_index: Option<Vec<String>>,
) -> HashMap<u32, AuthorDetailsV2> {
    let output: HashMap<u32, AuthorDetailsV2>;
    if commits_to_index.is_none() {
        output = extract_details_parallel(
            origin_file_path.clone(),
        ).await;
    } else {
        output = index_some_commits(
            origin_file_path.clone(),
            commits_to_index.unwrap(),
        ).await;
    }

    if !output.is_empty() && should_print {
        println!("Extracted details for file: {}", origin_file_path);
    }

    output
}
