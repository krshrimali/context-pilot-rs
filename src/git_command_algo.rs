use crate::config_impl;

use std::{
    path::Path,
    process::{Command, Stdio},
};

use crate::contextgpt_structs::AuthorDetails;

pub fn parse_str(input_str: &str, file_path: &str, end_line_number: usize) -> Vec<AuthorDetails> {
    let mut author_details_vec: Vec<AuthorDetails> = vec![];
    for split_line in input_str.split('\n') {
        if split_line.len() < 3 {
            continue;
        }
        let split_left_bracket: Vec<&str> = split_line.split('(').collect();
        let split_right_bracket: Vec<&str> = split_left_bracket
            .get(1)
            .expect("Expected a string but got none")
            .split(')')
            .collect();
        let left_split_vec: Vec<&str> = split_left_bracket.first().unwrap().split(' ').collect();
        let commit_hash = left_split_vec.first().unwrap();
        let author_details = AuthorDetails::serialize_from_str(
            split_right_bracket.first().unwrap().to_string(),
            commit_hash.to_string(),
            file_path,
            Vec::new(),
            end_line_number,
        );
        author_details_vec.push(author_details);
    }
    author_details_vec
}

pub fn get_files_for_commit_hash(commit_hash: &str) -> Vec<String> {
    let diff_command = Command::new("git")
        .args([
            "diff-tree",
            "--no-commit-id",
            "--name-only",
            commit_hash,
            "-r",
        ])
        .stdout(Stdio::piped())
        .output()
        .unwrap();
    let diff_buf = String::from_utf8(diff_command.stdout).unwrap();
    let mut out_vec: Vec<String> = vec![];
    for item in diff_buf.split('\n') {
        if item.is_empty() {
            continue;
        }
        out_vec.push(item.to_string());
    }
    out_vec
}

pub fn get_data_for_line(
    parsed_output: Vec<AuthorDetails>,
    start_line_number: usize,
    end_line_number: usize,
) -> Option<Vec<AuthorDetails>> {
    let mut output_list: Vec<AuthorDetails> = vec![];
    for output in parsed_output {
        if output.line_number >= start_line_number && output.line_number <= end_line_number {
            output_list.push(output);
        }
    }
    // TODO: Address when line number is not valid or found
    if output_list.is_empty() {
        None
    } else {
        Some(output_list)
    }
}

pub fn extract_details(
    start_line_number: usize,
    end_line_number: usize,
    file_path: String,
    config_obj: &config_impl::Config,
) -> Vec<AuthorDetails> {
    let mut binding = Command::new("git");
    let command = binding.args([
        "blame",
        "-L",
        &(start_line_number.to_string() + "," + &end_line_number.to_string()),
        "-w",
        "-M",
        "-C",
        "--",
        file_path.as_str(),
    ]);
    let output = command.stdout(Stdio::piped()).output().unwrap();
    let stdout_buf = String::from_utf8(output.stdout).unwrap();
    let parsed_output = parse_str(stdout_buf.as_str(), &file_path, end_line_number);

    let vec_author_detail_for_line =
        get_data_for_line(parsed_output, start_line_number, end_line_number);

    let mut result_author_details: Vec<AuthorDetails> = Vec::new();
    if vec_author_detail_for_line.is_none() {
        return result_author_details;
    }

    for author_detail_for_line in vec_author_detail_for_line.unwrap() {
        let val = author_detail_for_line;

        let mut commit_id = val.commit_hash;
        let out_files_for_commit_hash = get_files_for_commit_hash(&commit_id);
        let mut all_files_changed_initial_commit: Vec<String> = Vec::new();
        for each_file in out_files_for_commit_hash {
            let each_file_path = Path::new(&each_file);
            if !each_file_path.exists() {
                // Uhmm, either the file was moved - renamed - or deleted 🤔
                // NOTE: Deciding not to send this to the plugin, to avoid confusions...
                continue;
            }
            all_files_changed_initial_commit.push(each_file);
        }

        let mut blame_count: usize = 0;
        while blame_count != config_obj.commit_hashes_threshold {
            blame_count += 1;
            let line_string: String =
                val.line_number.to_string() + &','.to_string() + &val.line_number.to_string();
            let commit_url = commit_id.clone();
            // if !commit_url.ends_with('^') {
            //     commit_url = commit_id.clone() + "^";
            // }
            let cmd_args = vec![
                "blame",
                "-L",
                &line_string,
                "-w",
                "-M",
                &commit_url,
                "--",
                (file_path.as_str()),
            ];
            let new_blame_command = Command::new("git")
                .args(cmd_args.clone())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .unwrap();
            let out_buf = String::from_utf8(new_blame_command.stdout).unwrap();
            let parsed_buf = parse_str(out_buf.as_str(), &file_path, end_line_number);

            if let Some(valid_val) = get_data_for_line(parsed_buf, val.line_number, val.line_number)
            {
                commit_id = valid_val.get(0).unwrap().commit_hash.clone();
                let mut to_append_struct = valid_val.get(0).unwrap().clone();
                let out_files_for_commit_hash = get_files_for_commit_hash(&commit_id);
                let mut all_files_changed = Vec::new();
                for each_file in out_files_for_commit_hash {
                    let each_file_path = Path::new(&each_file);
                    if !each_file_path.exists() {
                        // NOTE: If file doesn't exist, maybe it was moved/renamed/deleted - so skip it for now
                        continue;
                    }
                    all_files_changed.push(each_file);
                }
                for each_initial_commit_file in all_files_changed_initial_commit.clone() {
                    if all_files_changed.contains(&each_initial_commit_file) {
                        continue;
                    }
                    all_files_changed.push(each_initial_commit_file);
                }
                to_append_struct.contextual_file_paths = all_files_changed;
                result_author_details.push(to_append_struct);
            }
        }
    }

    println!("Doing this for file path: {:?}", file_path);
    // for each_result in result_author_details.iter() {
    //     if each_result.origin_file_path == file_path {
    //         panic!("Wrong");
    //     }
    // }
    result_author_details
}
