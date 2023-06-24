use std::collections::BTreeMap;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::config::LAST_MANY_COMMIT_HASHES;
use crate::contextgpt_structs::AuthorDetails;

fn parse_str(input_str: &str, file_path: &str) -> Vec<AuthorDetails> {
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
        );
        author_details_vec.push(author_details);
    }
    author_details_vec
}

fn get_files_for_commit_hash(commit_hash: &str) -> Vec<String> {
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

fn get_data_for_line(
    parsed_output: Vec<AuthorDetails>,
    start_line_number: usize,
    end_line_number: usize,
) -> Vec<AuthorDetails> {
    let mut output_list: Vec<AuthorDetails> = vec![];
    for output in parsed_output {
        if output.line_number >= start_line_number && output.line_number <= end_line_number {
            output_list.push(output);
        }
    }
    // TOOD: Address when line number is not valid or found
    output_list
}

pub fn get_unique_files_changed(
    file_path: String,
    start_line_number: usize,
    end_line_number: usize,
) -> String {
    let mut binding = Command::new("git");
    let command = binding.args([
        "blame",
        "-L",
        &(start_line_number.to_string() + "," + &end_line_number.to_string()),
        "-w",
        "-M",
        "--",
        file_path.as_str(),
    ]);
    // println!("Command: {:?}", command);
    let output = command.stdout(Stdio::piped()).output().unwrap();
    let stdout_buf = String::from_utf8(output.stdout).unwrap();
    let parsed_output = parse_str(stdout_buf.as_str(), &file_path);

    let vec_author_detail_for_line =
        get_data_for_line(parsed_output, start_line_number, end_line_number);

    let mut all_files_changed: Vec<String> = Vec::new();
    for author_detail_for_line in vec_author_detail_for_line {
        let val = author_detail_for_line;

        let mut commit_id = val.commit_hash;
        let out_files_for_commit_hash = get_files_for_commit_hash(&commit_id);
        for each_file in out_files_for_commit_hash {
            let each_file_path = Path::new(&each_file);
            if !each_file_path.exists() {
                // Uhmm, either the file was moved - renamed - or deleted 🤔
                // NOTE: Deciding not to send this to the plugin, to avoid confusions...
                continue;
            }
            all_files_changed.push(each_file);

            // TODO: need to find an efficient way right now to fix this
            // let mut sanitized_file_path = each_file.clone();
            // // println!("Checking for {:?}", each_file);
            // if !each_file_path.exists() {
            //     sanitized_file_path = get_correct_file_path(&each_file);
            //     // println!("Sanitized: {:?}", sanitized_file_path);
            //     // println!("Path before: {:?}", each_file);
            // }
            // all_files_changed.push(sanitized_file_path);
        }

        let mut blame_count: i32 = 0;
        while blame_count != LAST_MANY_COMMIT_HASHES {
            blame_count += 1;
            let line_string: String =
                val.line_number.to_string() + &','.to_string() + &val.line_number.to_string();
            let commit_url = commit_id.clone() + "^";
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
                .output()
                .unwrap();
            let out_buf = String::from_utf8(new_blame_command.stdout).unwrap();
            let parsed_buf = parse_str(out_buf.as_str(), &file_path);
            let author_detail_for_line =
                get_data_for_line(parsed_buf, val.line_number, val.line_number);
            if author_detail_for_line.is_empty() {
                break;
            }
            let val = author_detail_for_line.get(0).unwrap();
            commit_id = val.commit_hash.clone();
            let out_files_for_commit_hash = get_files_for_commit_hash(&commit_id);
            for each_file in out_files_for_commit_hash {
                let each_file_path = Path::new(&each_file);
                if !each_file_path.exists() {
                    // NOTE: If file doesn't exist, maybe it was moved/renamed/deleted - so skip it for now
                    continue;
                }
                all_files_changed.push(each_file);
                // let mut sanitized_file_path = each_file.clone();
                // // println!("Checking for {:?}", each_file);
                // if !each_file_path.exists() {
                //     sanitized_file_path = get_correct_file_path(&each_file);
                //     //     println!("Sanitized: {:?}", sanitized_file_path);
                //     //     println!("Path before: {:?}", each_file);
                // }
                // all_files_changed.push(sanitized_file_path);
            }
        }
    }
    let sorted_map = all_files_changed
        .iter()
        .fold(BTreeMap::new(), |mut acc, c| {
            *acc.entry(c.to_string()).or_insert(0) += 1;
            acc
        });
    let mut output_result = sorted_map.keys().fold(String::new(), |mut res, val| {
        res.push_str(val);
        res.push(',');
        res
    });
    if output_result.ends_with(',') {
        output_result.pop();
    }
    output_result
}

pub fn parse_follow(input_str: &str, input_path: &str) -> Option<String> {
    let mut split_input_lines: Vec<&str> = input_str.split('\t').collect();
    split_input_lines.reverse();
    let mut just_two = 1;
    let mut final_path: Option<String> = None;
    for mut each_line in split_input_lines {
        if just_two < 0 {
            break;
        }
        if just_two != 1 {
            final_path = Some(each_line.to_string());
        }
        each_line = each_line.trim_end_matches('\n');
        // FIXME: use aboslute paths here instead
        if input_path.contains(each_line) || each_line.contains(input_path) || just_two != 1 {
            just_two -= 1;
        }
    }
    final_path
}

pub fn fix_details_in_case_of_move(vec_author_details: Vec<AuthorDetails>) -> Vec<AuthorDetails> {
    let mut output_vec: Vec<AuthorDetails> = Vec::new();
    for author_detail in vec_author_details {
        let file_path = std::path::Path::new(&author_detail.file_path);
        if !file_path.is_file() {
            output_vec.push(author_detail);
        } else {
            let output = Command::new("git")
                .args(["log", "--follow", "--raw", "-n 1", &author_detail.file_path])
                .stdout(Stdio::piped())
                .output()
                .unwrap();
            let stdout_buf = String::from_utf8(output.stdout).unwrap();
            let parsed_output = parse_follow(stdout_buf.as_str(), &author_detail.file_path);
            if let Some(final_path) = parsed_output {
                output_vec.push(AuthorDetails {
                    file_path: final_path,
                    commit_hash: author_detail.commit_hash,
                    author_full_name: author_detail.author_full_name,
                    line_number: author_detail.line_number,
                })
            };
        }
    }
    output_vec
}

fn parse_moved(output: &str, path_obj: &str) -> Option<String> {
    for each_file_combination_moved in output.split('\n') {
        let comb: Vec<&str> = each_file_combination_moved.split('\t').collect();
        if comb.is_empty() || comb.len() <= 1 {
            continue;
        }
        if comb.get(1).unwrap() == &path_obj {
            return Some(comb.get(2).unwrap().to_string());
        }
    }
    Some("".to_string())
}

pub fn _correct_file_path(path_obj: &Path) -> Option<String> {
    let output = Command::new("git")
        .args([
            "log",
            "--format=%h",
            "-m",
            "--first-parent",
            "--diff-filter=R",
            "--name-status",
            // "|",
            // "grep",
            // path_obj.to_str().unwrap(),
        ])
        .stdout(Stdio::piped())
        .output()
        .unwrap();
    // println!("output: {:?}", output);
    // println!("path: {:?}", path_obj.to_str().unwrap());
    let stdout_buf = String::from_utf8(output.stdout).unwrap();
    let parsed_output = parse_moved(stdout_buf.as_str(), path_obj.to_str().unwrap());
    if let Some(final_path) = parsed_output {
        return Some(final_path);
    }
    None
}

pub fn get_contextual_authors(
    file_path: String,
    start_line_number: usize,
    end_line_number: usize,
) -> String {
    let output = Command::new("git")
        .args([
            "blame",
            "-L",
            &(start_line_number.to_string() + "," + &end_line_number.to_string()),
            "-w",
            "-M",
            "-C",
            "--",
            file_path.as_str(),
        ])
        .stdout(Stdio::piped())
        .output()
        .unwrap();
    let stdout_buf = String::from_utf8(output.stdout).unwrap();
    let parsed_output = parse_str(stdout_buf.as_str(), &file_path);

    let vec_author_detail_for_line =
        get_data_for_line(parsed_output, start_line_number, end_line_number);
    // TODO: Use this function when files don't exist and have been moved/renamed
    // vec_author_detail_for_line = fix_details_in_case_of_move(vec_author_detail_for_line.clone());

    let mut author_details: Vec<String> = Vec::new();
    for author_detail_for_line in vec_author_detail_for_line {
        let val = author_detail_for_line;
        author_details.push(val.author_full_name);

        let mut commit_id = val.commit_hash;
        let mut blame_count: i32 = 0;
        while blame_count != LAST_MANY_COMMIT_HASHES {
            blame_count += 1;
            let line_string: String =
                val.line_number.to_string() + &','.to_string() + &val.line_number.to_string();
            let commit_url = commit_id.clone() + "^";
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
                .output()
                .unwrap();
            let out_buf = String::from_utf8(new_blame_command.stdout).unwrap();
            let parsed_buf = parse_str(out_buf.as_str(), &file_path);
            let author_detail_for_line =
                get_data_for_line(parsed_buf, val.line_number, val.line_number);
            if author_detail_for_line.is_empty() {
                break;
            }
            let val = author_detail_for_line.get(0).unwrap();
            commit_id = val.commit_hash.clone();
            author_details.push(val.author_full_name.clone());
        }
    }
    let sorted_map = author_details.iter().fold(BTreeMap::new(), |mut acc, c| {
        *acc.entry(c.to_string()).or_insert(0) += 1;
        acc
    });
    let reverse_sorted_map: BTreeMap<&i32, &String> =
        sorted_map.iter().map(|(k, v)| (v, k)).collect();
    let mut res = reverse_sorted_map
        .values()
        .fold(String::new(), |mut res, val| {
            res.push_str(val);
            res.push(',');
            res
        });
    if res.ends_with(',') {
        res.pop();
    }
    res
}
