use std::collections::HashMap;

use crate::{contextgpt_structs::AuthorDetailsV2, git_command_algo::extract_details_parallel};

pub async fn perform_for_whole_file(
    origin_file_path: String,
    should_print: bool,
) -> HashMap<u32, AuthorDetailsV2> {
    let output = extract_details_parallel(
        // 1 as usize,
        // end_line_number as usize,
        origin_file_path.clone(),
    ).await;


    if !output.is_empty() && should_print {
        println!("Extracted details for file: {}", origin_file_path);
    }

    output
}
