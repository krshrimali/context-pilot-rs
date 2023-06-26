use std::collections::HashMap;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::config::LAST_MANY_COMMIT_HASHES;
use crate::contextgpt_structs::AuthorDetails;
use crate::db::DB;
use crate::git_command_algo::{extract_details, get_data_for_line, parse_str};

pub fn get_unique_files_changed(
    origin_file_path: String,
    start_line_number: usize,
    end_line_number: usize,
    db_obj: &mut DB,
) -> String {
    let configured_file_path: String =
        format!("{origin_file_path}**{start_line_number}**{end_line_number}");
    // Check in the DB first
    let mut res = String::new();
    let mut visited: HashMap<String, usize> = HashMap::new();
    if let Some(obj) = db_obj.exists(&configured_file_path) {
        for author_detail in obj {
            if visited.contains_key(&author_detail.origin_file_path) {
                continue;
            }
            visited.insert(author_detail.origin_file_path.clone(), 1);
            res.push_str(&author_detail.origin_file_path);
            res.push(',');
        }
        if res.ends_with(',') {
            res.pop();
        }
        return res;
    }
    // INSERT HERE
    let output = extract_details(start_line_number, end_line_number, origin_file_path);
    let mut res: HashMap<String, usize> = HashMap::new();
    for single_struct in output {
        db_obj.append(&configured_file_path, single_struct.clone());
        if res.contains_key(&single_struct.origin_file_path) {
            let count = res.get(&single_struct.origin_file_path).unwrap() + 1;
            res.insert(single_struct.origin_file_path, count);
            continue;
        }
        res.insert(single_struct.origin_file_path, 0);
    }
    db_obj.store();
    let mut res_string: String = String::new();
    for key in res.keys() {
        if key.contains("Commited Yet") {
            continue;
        }
        res_string.push_str(key.as_str());
        res_string.push(',');
    }
    if res_string.ends_with(',') {
        res_string.pop();
    }
    res_string
}

pub fn get_contextual_authors(
    file_path: String,
    start_line_number: usize,
    end_line_number: usize,
    db_obj: &mut DB,
) -> String {
    let configured_file_path: String =
        format!("{file_path}**{start_line_number}**{end_line_number}");
    // Check in the DB first
    let mut res = String::new();
    let mut visited: HashMap<String, usize> = HashMap::new();
    if let Some(obj) = db_obj.exists(&configured_file_path) {
        for author_detail in obj {
            if visited.contains_key(&author_detail.author_full_name) {
                continue;
            }
            if author_detail.author_full_name.contains("Not Committed Yet") {
                continue;
            }
            visited.insert(author_detail.author_full_name.clone(), 1);
            res.push_str(&author_detail.author_full_name);
            res.push(',');
        }
        if res.ends_with(',') {
            res.pop();
        }
        return res;
    }
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
        get_data_for_line(parsed_output, start_line_number, end_line_number).unwrap_or(Vec::new());

    let mut author_details: Vec<AuthorDetails> = Vec::new();
    for author_detail_for_line in vec_author_detail_for_line {
        author_details.push(author_detail_for_line.clone());

        let mut commit_id = author_detail_for_line.clone().commit_hash;
        let mut blame_count: i32 = 0;
        while blame_count != LAST_MANY_COMMIT_HASHES {
            blame_count += 1;
            let line_string: String = author_detail_for_line.line_number.to_string()
                + &','.to_string()
                + &author_detail_for_line.line_number.to_string();
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

            if let Some(valid_val) = get_data_for_line(
                parsed_buf,
                author_detail_for_line.line_number,
                author_detail_for_line.line_number,
            ) {
                commit_id = valid_val.get(0).unwrap().commit_hash.clone();
                author_details.push(author_detail_for_line.clone());
            }
        }
    }

    let mut res: HashMap<String, usize> = HashMap::new();
    for author_detail_val in author_details {
        db_obj.append(&configured_file_path, author_detail_val.clone());
        if res.contains_key(&author_detail_val.author_full_name) {
            let count = res.get(&author_detail_val.author_full_name).unwrap() + 1;
            res.insert(author_detail_val.author_full_name, count);
            continue;
        }
        res.insert(author_detail_val.author_full_name, 0);
    }
    db_obj.store();
    let mut res_string: String = String::new();
    for key in res.keys() {
        if key.contains("Not Committed Yet") {
            continue;
        }
        res_string.push_str(key.as_str());
        res_string.push(',');
    }
    res_string
}
