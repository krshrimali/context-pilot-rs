use crate::contextgpt_structs::AuthorDetailsV2;

use std::{
    fs::File,
    io::{BufRead, BufReader},
    process::{Command, Stdio},
};


pub fn get_files_changed(commit_hash: &str) -> Vec<String> {
    // Use git show (minimal) API to find "all the files" changed in the given commit hash.
    // git show --pretty="" --name-only <commit_hash>
    let mut command = Command::new("git");
    let c_hash = commit_hash.strip_suffix("'").unwrap();
    command.args([
        "show",
        "--pretty=",
        "--name-only",
        c_hash,
    ]);
    let output = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap();
    let stdout_buf = String::from_utf8(output.stdout).unwrap();
    let mut files_changed: Vec<String> = Vec::new();
    for line in stdout_buf.lines() {
        files_changed.push(line.to_string());
    }
    files_changed
}


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
            "--pretty='A:%an|H:%h'",
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
