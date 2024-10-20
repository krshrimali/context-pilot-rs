use linecount::count_lines;

use crate::config_impl;
use crate::contextgpt_structs::AuthorDetails;
use crate::db::DB;
use crate::git_command_algo::extract_details;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

fn split_output_and_create_map(
    output_single_line: String,
    visited_count_map: &mut HashMap<String, usize>,
    origin_file_path: &String,
    res_string: &mut String,
) {
    for single_string_from_output in output_single_line.split(',') {
        // if single_string_from_output == origin_file_path || single_string_from_output.is_empty() {
        //     continue;
        // }
        if single_string_from_output.is_empty() {
            continue;
        }
        if visited_count_map.contains_key(single_string_from_output) {
            visited_count_map.insert(
                single_string_from_output.to_string().clone(),
                visited_count_map.get(single_string_from_output).unwrap() + 1,
            );
            continue;
        }
        visited_count_map.insert(single_string_from_output.to_string().clone(), 1);
        res_string.push_str(single_string_from_output);
        res_string.push(',');
    }
}

pub fn get_unique_files_changed(
    origin_file_path: String,
    start_line_number: &usize,
    end_line_number: &usize,
    db_obj: &mut DB,
    config_obj: &config_impl::Config,
) -> String {
    // Check in the DB first
    let mut res = String::new();
    let mut visited_count_map: HashMap<String, usize> = HashMap::new();
    let (existing_result_optional, unvisited_indices) =
        db_obj.exists_and_return(&origin_file_path, start_line_number, end_line_number);
    match existing_result_optional {
        Some(existing_result) => {
            for author_detail in existing_result {
                for each_file in author_detail.contextual_file_paths.clone() {
                    if each_file == origin_file_path {
                        continue;
                    }
                    if visited_count_map.contains_key(&each_file) {
                        visited_count_map.insert(
                            each_file.clone(),
                            visited_count_map.get(&each_file).unwrap() + 1,
                        );
                        continue;
                    }
                    visited_count_map.insert(each_file.clone(), 0);
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
                for each_unvisited_index in unvisited_indices.iter() {
                    // let is_end = (*each_unvisited_index as usize) == (unvisited_indices.len() - 1);
                    let output_single_line = perform_for_single_line(
                        *each_unvisited_index as usize,
                        *each_unvisited_index as usize,
                        origin_file_path.clone(),
                        db_obj,
                        false,
                        config_obj,
                    );
                    split_output_and_create_map(
                        output_single_line,
                        &mut visited_count_map,
                        &origin_file_path,
                        &mut res_string,
                    );
                }
                if res_string.ends_with(',') {
                    let _ = res_string.pop();
                }
                return config_impl::trim_result(res_string, config_obj.file_count_threshold);
            }
            config_impl::trim_result(res, config_obj.file_count_threshold)
        }
        None => {
            let mut final_result = "".to_string();
            for each_unvisited_index in unvisited_indices.iter() {
                // let is_end = (*each_unvisited_index as usize) == (unvisited_indices.len() - 1);
                let output_single_line = perform_for_single_line(
                    *each_unvisited_index as usize,
                    *each_unvisited_index as usize,
                    origin_file_path.clone(),
                    db_obj,
                    false,
                    config_obj,
                );
                split_output_and_create_map(
                    output_single_line,
                    &mut visited_count_map,
                    &origin_file_path,
                    &mut final_result,
                );
            }
            if final_result.ends_with(',') {
                let _ = final_result.pop();
            }
            config_impl::trim_result(final_result, config_obj.file_count_threshold)
        }
    }
}

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
    // println!(
    //     "db file path while performing for whole file: {}",
    //     curr_db.db_file_path.clone()
    // );
    // let mut res: HashMap<String, usize> = HashMap::new();

    let start_line_number = 1;
    let file = std::fs::File::open(origin_file_path.clone()).unwrap();
    // let end_line_number = 1000000;
    let end_line_number = count_lines(file).unwrap() as i32 - 1;
    let output = extract_details(
        start_line_number,
        end_line_number as usize,
        origin_file_path.clone(),
        config_obj,
    );
    output
    // db_obj.append(
    //     &origin_file_path,
    //     start_line_number,
    //     end_line_number,
    //     output.clone(), // TODO: don't clone everywhere!!
    // );

    // FIXME: DO NOT STORE.
    // We should instead persist db_obj somehow.
    // db_obj.store();
    // extract_string_from_output(output, is_author_mode)
}

pub fn perform_for_single_line(
    start_line_number: usize,
    end_line_number: usize,
    origin_file_path: String,
    db_obj: &mut DB,
    is_author_mode: bool,
    config_obj: &config_impl::Config,
) -> String {
    let output = extract_details(
        start_line_number,
        end_line_number,
        origin_file_path.clone(),
        config_obj,
    );
    // println!(
    //     "Only computing for {:?} -> {:?}",
    //     start_line_number, end_line_number
    // );
    let mut res: HashMap<String, usize> = HashMap::new();
    db_obj.append(&origin_file_path, start_line_number, output.clone());
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
    config_obj: &config_impl::Config,
) -> String {
    // Check in the DB first
    let mut res = String::new();
    let mut visited_count_map: HashMap<String, usize> = HashMap::new();
    let (existing_result_optional, unvisited_indices) =
        db_obj.exists_and_return(&origin_file_path, start_line_number, end_line_number);
    match existing_result_optional {
        Some(existing_result) => {
            for author_detail in existing_result {
                if author_detail.author_full_name.contains("Not Committed Yet") {
                    continue;
                }
                if visited_count_map.contains_key(&author_detail.author_full_name) {
                    visited_count_map.insert(
                        author_detail.author_full_name.clone(),
                        visited_count_map
                            .get(&author_detail.author_full_name)
                            .unwrap_or(&0)
                            + 1,
                    );
                    continue;
                }
                visited_count_map.insert(author_detail.author_full_name.clone(), 1);
                res.push_str(&author_detail.author_full_name);
                res.push(',');
            }
            if res.ends_with(',') {
                res.pop();
            }
            if !unvisited_indices.is_empty() {
                // find if multiple splits are there
                let mut res_string: String = res;
                for each_unvisited_index in unvisited_indices.iter() {
                    let output_single_line = perform_for_single_line(
                        *each_unvisited_index as usize,
                        *each_unvisited_index as usize,
                        origin_file_path.clone(),
                        db_obj,
                        true,
                        config_obj,
                    );
                    split_output_and_create_map(
                        output_single_line,
                        &mut visited_count_map,
                        &origin_file_path,
                        &mut res_string,
                    );
                }
                if res_string.ends_with(',') {
                    let _ = res_string.pop();
                }
                return config_impl::trim_result(res_string, config_obj.file_count_threshold);
            }
            config_impl::trim_result(res, config_obj.file_count_threshold)
        }
        None => {
            let mut final_result = "".to_string();
            for each_unvisited_index in unvisited_indices.iter() {
                let output_single_line = perform_for_single_line(
                    *each_unvisited_index as usize,
                    *each_unvisited_index as usize,
                    origin_file_path.clone(),
                    db_obj,
                    true,
                    config_obj,
                );
                split_output_and_create_map(
                    output_single_line,
                    &mut visited_count_map,
                    &origin_file_path,
                    &mut final_result,
                );
            }
            if final_result.ends_with(',') {
                let _ = final_result.pop();
            }
            config_impl::trim_result(final_result, config_obj.file_count_threshold)
        }
    }
}
