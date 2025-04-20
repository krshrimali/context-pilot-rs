use crate::{config, config_impl, contextgpt_structs::AuthorDetails};

use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
    path::Path,
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    thread,
};

pub fn parse_str(input_str: &str, file_path: &str, end_line_number: usize) -> Vec<AuthorDetails> {
    let mut author_details_vec: Vec<AuthorDetails> = Vec::new();

    for line in input_str.lines() {
        if line.trim().len() < 3 {
            continue;
        }

        // Split on the first '('
        let (left_part, right_part) = match line.split_once('(') {
            Some((left, right)) => (left.trim(), right),
            None => continue,
        };

        // Split on the first ')'
        let author_str = match right_part.split_once(')') {
            Some((author, _)) => author.trim(),
            None => continue,
        };

        let commit_hash = match left_part.split_whitespace().next() {
            Some(hash) => hash,
            None => continue,
        };

        let author_details = AuthorDetails::serialize_from_str(
            author_str.to_string(),
            commit_hash.to_string(),
            file_path,
            Vec::new(),
            end_line_number,
        );

        author_details_vec.push(author_details);
    }

    author_details_vec
}

// pub fn parse_str_(
//     input_str: &str,
//     file_path: &str,
//     line_number: usize,
//     end_line_number: usize,
// ) -> Vec<AuthorDetails> {
//     let mut author_details_vec: Vec<AuthorDetails> = Vec::new();
//
//     for line in input_str.split('\n') {
//         // Format: "A:<Author Name> H:<Commit Hash>"
//         // Example: "A:John Doe H:abc1234"
//         let parts: Vec<&str> = line.split('|').collect();
//         if parts.len() < 2 {
//             continue; // Skip lines that don't have the expected format
//         }
//         let author_str = parts[0].trim().replace("A:", "");
//         let commit_hash = parts[1].trim().replace("H:", "");
//
//         let author_details = AuthorDetails::serialize_from_str(
//             author_str.to_string(),
//             commit_hash.to_string(),
//             file_path,
//             Vec::new(),
//             line_number,
//             end_line_number,
//         );
//
//         author_details_vec.push(author_details);
//     }
//
//     author_details_vec
// }

pub fn get_files_for_commit_hash(commit_hash: &str) -> Vec<String> {
    let diff_command = Command::new("git")
        .args(["show", "--name-only", "--pretty=''", commit_hash])
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

pub fn extract_details(file_path: String) -> Vec<AuthorDetails> {
    // git blame the whole file, group by commits, and then for each commit -> go back
    // config_impl.threshold commits back,
    // and get the files changed in that commit -> store them in AuthorDetails.
    let config_obj: config_impl::Config = config_impl::read_config(config::CONFIG_FILE_NAME);
    let output = Command::new("git")
        .args(["blame", "-n", &file_path])
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to run git blame")
        .stdout
        .expect("Failed to capture stdout");

    let reader = BufReader::new(output);
    let mut blame_map: HashMap<String, Vec<String>> = HashMap::new();

    for line_result in reader.lines() {
        if let Ok(line) = line_result {
            if let Some((hash, content)) = line.split_once(' ') {
                // Trim content to fetch author name and the line numbers as well.
                // Format would be like:
                // 100 (Kushashwa Ravi shrimali ...)
                // We only need the line number initially and the author name.
                let line_number = content.split('(').next().unwrap_or("").trim();
                println!("Line number: {:?}", line_number);
                let content = content.split('(').nth(1).unwrap_or("").trim();
                let check = content.split(')').next().unwrap_or("").trim();
                println!("Author name: {:?}", check);
            }
        }
    }

    // for (hash, lines) in &blame_map {
    //     println!("Commit {}:", hash);
    //     // Now parse the line
    //     println!("{:?}", lines);
    // }

    Vec::new()
}

// pub fn extract_details(
//     start_line_number: usize,
//     end_line_number: usize,
//     file_path: String,
// ) -> Vec<AuthorDetails> {
//     let config_obj: config_impl::Config = config_impl::read_config(config::CONFIG_FILE_NAME);
//     let mut output_auth_details: Vec<AuthorDetails> = Vec::new();
//     // use git log -Lstart_line_number,end_line_number:file_path --no-patch
//     for i in start_line_number..=end_line_number {
//         let mut command = Command::new("git".to_string());
//         command.args([
//             "log",
//             "-L",
//             format!("{},{}:{}", i, i, &file_path).as_str(),
//             "-n",
//             config_obj.commit_hashes_threshold.to_string().as_str(),
//             "--no-merges",
//             "--pretty='A:%an|H:%H'",
//             "--diff-filter=AM", // Added and Modified
//             "--no-patch",
//         ]);
//         let output = command
//             .stdout(Stdio::piped())
//             .stderr(Stdio::piped())
//             .output()
//             .unwrap();
//         let stdout_buf = String::from_utf8(output.stdout).unwrap();
//         // let stderr_buf = String::from_utf8(output.stderr).unwrap();
//         let parsed_output = parse_str(&stdout_buf, &file_path, start_line_number, end_line_number);
//         let append_contextual_paths_to_struct = |mut author_details: AuthorDetails| {
//             let commit_id = author_details.commit_hash.clone();
//             let out_files_for_commit_hash = get_files_for_commit_hash(&commit_id);
//             let all_files_changed: Vec<String> = out_files_for_commit_hash
//                 .into_iter()
//                 .filter(|f| Path::new(f).exists())
//                 .collect();
//             author_details.contextual_file_paths = all_files_changed;
//             author_details
//         };
//         let parsed_output: Vec<AuthorDetails> = parsed_output
//             .into_iter()
//             .map(append_contextual_paths_to_struct)
//             .collect();
//         output_auth_details.extend(parsed_output);
//     }
//     output_auth_details
// }

// pub fn extract_details(
//     start_line_number: usize,
//     end_line_number: usize,
//     file_path: String,
// ) -> Vec<AuthorDetails> {
//     // let config_obj = config_impl::Config::default();
//     let config_obj: config_impl::Config = config_impl::read_config(config::CONFIG_FILE_NAME);
//     let output = Command::new("git")
//         .args([
//             "blame",
//             "-L",
//             &(start_line_number.to_string() + "," + &end_line_number.to_string()),
//             "-w",
//             "-M",
//             "-C",
//             "--",
//             &file_path,
//         ])
//         .stdout(Stdio::piped())
//         .output()
//         .unwrap();
//
//     let stdout_buf = String::from_utf8(output.stdout).unwrap();
//     let parsed_output = parse_str(&stdout_buf, &file_path, end_line_number);
//
//     let vec_author_detail_for_line =
//         get_data_for_line(parsed_output, start_line_number, end_line_number);
//
//     let result_author_details: Arc<Mutex<Vec<AuthorDetails>>> = Arc::new(Mutex::new(Vec::new()));
//
//     if vec_author_detail_for_line.is_none() {
//         return Vec::new();
//     }
//
//     let mut handles = vec![];
//
//     for val in vec_author_detail_for_line.unwrap() {
//         let file_path = file_path.clone();
//         let result_author_details = Arc::clone(&result_author_details);
//
//         let handle = thread::spawn(move || {
//             let mut commit_id = val.commit_hash.clone();
//             let out_files_for_commit_hash = get_files_for_commit_hash(&commit_id);
//             let all_files_changed_initial_commit: Vec<String> = out_files_for_commit_hash
//                 .into_iter()
//                 .filter(|f| Path::new(f).exists())
//                 .collect();
//
//             let mut blame_count = 0;
//
//             while blame_count != config_obj.commit_hashes_threshold {
//                 blame_count += 1;
//
//                 let line_string = format!("{},{}", val.line_number, val.line_number);
//
//                 let cmd_args = vec![
//                     "blame",
//                     "-L",
//                     &line_string,
//                     "-w",
//                     "-M",
//                     &commit_id,
//                     "--",
//                     &file_path,
//                 ];
//
//                 let new_blame_command = Command::new("git")
//                     .args(&cmd_args)
//                     .stdout(Stdio::piped())
//                     .stderr(Stdio::piped())
//                     .output()
//                     .unwrap();
//
//                 let out_buf = String::from_utf8(new_blame_command.stdout).unwrap();
//                 let parsed_buf = parse_str(&out_buf, &file_path, val.end_line_number);
//
//                 if let Some(valid_val) =
//                     get_data_for_line(parsed_buf, val.line_number, val.line_number)
//                 {
//                     commit_id = valid_val[0].commit_hash.clone();
//                     let mut to_append_struct = valid_val[0].clone();
//
//                     let out_files_for_commit_hash = get_files_for_commit_hash(&commit_id);
//                     let mut all_files_changed: Vec<String> = out_files_for_commit_hash
//                         .into_iter()
//                         .filter(|f| Path::new(f).exists())
//                         .collect();
//
//                     for each_initial_commit_file in &all_files_changed_initial_commit {
//                         if !all_files_changed.contains(each_initial_commit_file) {
//                             all_files_changed.push(each_initial_commit_file.clone());
//                         }
//                     }
//
//                     to_append_struct.contextual_file_paths = all_files_changed;
//
//                     result_author_details.lock().unwrap().push(to_append_struct);
//                 }
//             }
//         });
//
//         handles.push(handle);
//     }
//
//     for handle in handles {
//         handle.join().unwrap();
//     }
//
//     Arc::try_unwrap(result_author_details)
//         .unwrap()
//         .into_inner()
//         .unwrap()
// }
