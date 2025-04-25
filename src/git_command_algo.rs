use crate::contextgpt_structs::AuthorDetailsV2;

use std::{
    fs::File,
    io::{BufRead, BufReader},
    process::{Command, Stdio},
};

pub fn parse_git_log_l(input_str: &str) -> AuthorDetailsV2 {
    // Ouptut is similar to:
    //A:Kushashwa Ravi Shrimali|H:84ada44f9980535f719803f009401b68b0b7336d
    //A:Kushashwa Ravi Shrimali|H:d75dc7ec45ba19cf4d5a6647246ca7c059c0ae0d
    let mut vec_auth_details: AuthorDetailsV2 = AuthorDetailsV2 {
        origin_file_path: String::new(),
        line_number: 1,
        commit_hashes: Vec::new(),
        author_full_name: Vec::new(),
    };
    for line in input_str.lines() {
        if line.trim().len() < 3 {
            continue;
        }

        // Split on the first '|'
        let (left_part, right_part) = match line.split_once('|') {
            Some((left, right)) => (left.trim(), right),
            None => continue,
        };

        // Split on the first ':'
        let author_str = match left_part.split_once(':') {
            Some((author, _)) => author.trim(),
            None => continue,
        };

        let commit_hash = match right_part.split_once(':') {
            Some(hash) => hash.1.trim(),
            None => continue,
        };
        let mut hashes = Vec::new();
        hashes.push(commit_hash.to_string());

        vec_auth_details.commit_hashes.push(commit_hash.to_string());
        vec_auth_details
            .author_full_name
            .push(author_str.to_string());
    }
    vec_auth_details
}

pub fn extract_details(file_path: String) -> Vec<AuthorDetailsV2> {
    // Run git log -L start_line_number,end_line_number:file_path --reverse to get the evolution of
    // the particular lines.
    // Find start line number and end line number of the file path.
    let start_line_number = 1;
    // Get the last line number by reading the file path
    let end_line_number = File::open(&file_path)
        .map(|file| {
            let reader = BufReader::new(file);
            reader.lines().count()
        })
        .unwrap_or(0);
    let mut final_output: Vec<AuthorDetailsV2> = Vec::new();
    for i in start_line_number..=end_line_number {
        let mut command = Command::new("git".to_string());
        command.args([
            "log",
            "-L",
            format!("{},{}:{}", i, i, &file_path).as_str(),
            // "-n",
            // "1",
            "--no-merges",
            "--pretty='A:%an|H:%H'",
            "--diff-filter=AM", // Added and Modified
            "--no-patch",
            "--reverse",
        ]);
        let output = command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .unwrap();
        let stdout_buf = String::from_utf8(output.stdout).unwrap();
        let mut parsed_output = parse_git_log_l(&stdout_buf);
        if parsed_output.commit_hashes.is_empty() {
            continue;
        }
        parsed_output.line_number = i;
        parsed_output.origin_file_path = file_path.clone();
        final_output.push(parsed_output);
    }
    final_output
}

// pub fn extract_details(file_path: String) -> Vec<AuthorDetails> {
//     // git blame the whole file, group by commits, and then for each commit -> go back
//     // config_impl.threshold commits back,
//     // and get the files changed in that commit -> store them in AuthorDetails.
//     let config_obj: config_impl::Config = config_impl::read_config(config::CONFIG_FILE_NAME);
//     let output = Command::new("git")
//         .args(["blame", "-n", &file_path])
//         .stdout(Stdio::piped())
//         .spawn()
//         .expect("Failed to run git blame")
//         .stdout
//         .expect("Failed to capture stdout");
//
//     let reader = BufReader::new(output);
//     let mut blame_map: HashMap<String, Vec<String>> = HashMap::new();
//     let vec_author_details: Vec<AuthorDetails> = Vec::new();
//
//     for line_result in reader.lines() {
//         if let Ok(line) = line_result {
//             if let Some((hash, content)) = line.split_once(' ') {
//                 // Trim content to fetch author name and the line numbers as well.
//                 // Format would be like:
//                 // 100 (Kushashwa Ravi shrimali ...)
//                 // We only need the line number initially and the author name.
//                 let line_number = content.split('(').next().unwrap_or("").trim();
//                 let content = content.split('(').nth(1).unwrap_or("").trim();
//                 let auth_detail_unfiltered = content.split(')').next().unwrap_or("").trim();
//                 // TODO: Add auth_name only and filter out everything else.
//                 blame_map.entry(hash.to_string())
//                     .or_insert_with(Vec::new)
//                     .push(format!("{} {}", line_number, auth_detail_unfiltered));
//             }
//         }
//     }
//
//     // Now you'd want to iterate over the blame_map and for each commit hash,
//     // and get the line numbers (min and max) - so as we get the range, and then we store the data.
//     let mut output_map_range: HashMap<u32, String> = HashMap::new();
//     blame_map.iter().for_each(|(hash, lines)| {
//         // For each commit hash, we can get the line numbers and then
//         // get the files changed in that commit.
//         let mut line_numbers: Vec<usize> = vec![];
//         for line in lines {
//             if let Some(line_number) = line.split_whitespace().next() {
//                 if let Ok(num) = line_number.parse::<usize>() {
//                     line_numbers.push(num);
//                 }
//             }
//         }
//         if !line_numbers.is_empty() {
//             // let min_line = *line_numbers.iter().min().unwrap();
//             let max_line = *line_numbers.iter().max().unwrap();
//             output_map_range.insert(max_line as u32, format!("{}", hash));
//         }
//     });
//     Vec::new()
// }

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
