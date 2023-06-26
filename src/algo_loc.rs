use std::collections::HashMap;
use crate::git_command_algo::extract_details;
use crate::db::DB;

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
            for each_file in author_detail.contextual_file_paths.clone() {
                if visited.contains_key(&each_file) {
                    continue;
                }
                visited.insert(each_file.clone(), 1);
                res.push_str(&each_file);
                res.push(',');
            }
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
        for each_file in single_struct.contextual_file_paths {
            if res.contains_key(&each_file) {
                let count = res.get(&each_file).unwrap() + 1;
                res.insert(each_file, count);
                continue;
            }
            res.insert(each_file, 0);
        }
    }
    db_obj.store();
    let mut res_string: String = String::new();
    for key in res.keys() {
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
    let output = extract_details(start_line_number, end_line_number, file_path);
    let mut res: HashMap<String, usize> = HashMap::new();
    for single_struct in output {
        db_obj.append(&configured_file_path, single_struct.clone());
        let author_full_name = single_struct.author_full_name;
        if res.contains_key(&author_full_name) {
            let count = res.get(&author_full_name).unwrap() + 1;
            res.insert(author_full_name, count);
            continue;
        }
        res.insert(author_full_name, 0);
    }
    db_obj.store();
    let mut res_string: String = String::new();
    for key in res.keys() {
        res_string.push_str(key.as_str());
        res_string.push(',');
    }
    if res_string.ends_with(',') {
        res_string.pop();
    }
    res_string
}
