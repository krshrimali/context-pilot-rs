use crate::db::DB;
use crate::git_command_algo::extract_details;
use std::collections::HashMap;

pub fn get_unique_files_changed(
    origin_file_path: String,
    start_line_number: &usize,
    end_line_number: &usize,
    db_obj: &mut DB,
) -> String {
    // Check in the DB first
    let mut res = String::new();
    let mut visited: HashMap<String, usize> = HashMap::new();
    let (existing_result_optional, unvisited_indices) =
        db_obj.exists_and_return(&origin_file_path, start_line_number, end_line_number);
    match existing_result_optional {
        Some(existing_result) => {
            for author_detail in existing_result {
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
            if !unvisited_indices.is_empty() {
                // find if multiple splits are there
                let mut res_string: String = res;
                for each_unvisited_index in unvisited_indices {
                    res_string += &perform_for_single_line(
                        each_unvisited_index,
                        each_unvisited_index,
                        origin_file_path.clone(),
                        db_obj,
                        false,
                        res_string.clone(),
                    );
                }
                if res_string.ends_with(',') {
                    let _ = res_string.pop();
                }
                return res_string;
            }
            res
        }
        None => {
            let mut final_result = "".to_string();
            for each_unvisited_index in unvisited_indices {
                final_result += &perform_for_single_line(
                    each_unvisited_index,
                    each_unvisited_index,
                    origin_file_path.clone(),
                    db_obj,
                    false,
                    final_result.clone(),
                );
            }
            if final_result.ends_with(',') {
                let _ = final_result.pop();
            }
            final_result
        }
    }
}

pub fn perform_for_single_line(
    start_line_number: usize,
    end_line_number: usize,
    origin_file_path: String,
    db_obj: &mut DB,
    is_author_mode: bool,
    current_output: String,
) -> String {
    let output = extract_details(start_line_number, end_line_number, origin_file_path.clone());
    // println!(
    //     "Only computing for {:?} -> {:?}",
    //     start_line_number, end_line_number
    // );
    let mut res: HashMap<String, usize> = HashMap::new();
    db_obj.append(
        &origin_file_path,
        start_line_number,
        end_line_number,
        output.clone(),
    );
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
    db_obj.store();
    let mut res_string: String = String::new();
    for key in res.keys() {
        if current_output.contains(key) {
            continue;
        }
        res_string.push_str(key.as_str());
        res_string.push(',');
    }
    res_string
}

pub fn get_contextual_authors(
    origin_file_path: String,
    start_line_number: &usize,
    end_line_number: &usize,
    db_obj: &mut DB,
) -> String {
    // Check in the DB first
    let mut res = String::new();
    let mut visited: HashMap<String, usize> = HashMap::new();
    let (existing_result_optional, unvisited_indices) =
        db_obj.exists_and_return(&origin_file_path, start_line_number, end_line_number);
    match existing_result_optional {
        Some(existing_result) => {
            for author_detail in existing_result {
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
            if !unvisited_indices.is_empty() {
                // find if multiple splits are there
                let mut res_string: String = res;
                for each_unvisited_index in unvisited_indices {
                    res_string += &perform_for_single_line(
                        each_unvisited_index,
                        each_unvisited_index,
                        origin_file_path.clone(),
                        db_obj,
                        true,
                        res_string.clone(),
                    );
                }
                if res_string.ends_with(',') {
                    let _ = res_string.pop();
                }
                return res_string;
            }
            res
        }
        None => {
            let mut final_result = "".to_string();
            for each_unvisited_index in unvisited_indices {
                final_result += &perform_for_single_line(
                    each_unvisited_index,
                    each_unvisited_index,
                    origin_file_path.clone(),
                    db_obj,
                    true,
                    final_result.clone(),
                );
            }
            if final_result.ends_with(',') {
                let _ = final_result.pop();
            }
            final_result
        }
    }
}
