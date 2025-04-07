use linecount::count_lines;

use crate::config_impl;
use crate::contextgpt_structs::AuthorDetails;
use crate::git_command_algo::extract_details;
use std::collections::HashMap;

pub fn extract_string_from_output(output: Vec<AuthorDetails>, is_author_mode: bool) -> String {
    let mut res: HashMap<String, usize> = HashMap::new();
    for single_struct in output {
        if is_author_mode {
            if res.contains_key(&single_struct.author_full_name) {
                let count = res.get(&single_struct.author_full_name).unwrap() + 1;
                res.insert(single_struct.author_full_name, count);
                continue;
            } else {
                res.insert(single_struct.author_full_name, 0);
            }
        } else {
            for each_file in single_struct.contextual_file_paths {
                if res.contains_key(&each_file) {
                    let count = res.get(&each_file).unwrap() + 1;
                    res.insert(each_file, count);
                } else {
                    res.insert(each_file, 0);
                }
            }
        }
    }
    // db_obj.store();
    let mut res_string: String = String::new();
    for key in res.keys() {
        res_string.push_str(key.as_str());
        res_string.push(',');
    }
    res_string
}

pub fn perform_for_whole_file(
    origin_file_path: String,
    config_obj: &config_impl::Config,
) -> Vec<AuthorDetails> {
    let file = match std::fs::File::open(&origin_file_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to open file '{}': {}", origin_file_path, e);
            return vec![];
        }
    };

    let end_line_number = match count_lines(file) {
        Ok(count) => count.saturating_sub(1) as i32,
        Err(e) => {
            eprintln!("Failed to count lines in '{}': {}", origin_file_path, e);
            return vec![];
        }
    };

    let output = extract_details(
        1,
        end_line_number as usize,
        origin_file_path.clone(),
        config_obj,
    );

    if output.is_empty() {
        // Do nothing for now!
        // eprintln!(
        //     "No author details found for '{}'. It may be a binary file, empty, or ignored.",
        //     origin_file_path
        // );
    }

    output
}
