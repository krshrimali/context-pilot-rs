use crate::{contextgpt_structs::AuthorDetailsV2, diff_v2};

use crate::git_command_algo;
use std::collections::{HashMap, HashSet};
use std::{
    process::{Command, Stdio},
};

pub fn get_files_changed(commit_hash: &str) -> Vec<String> {
    // Use git show (minimal) API to find "all the files" changed in the given commit hash.
    // git show --pretty="" --name-only <commit_hash>
    let mut command = Command::new("git");
    let c_hash = commit_hash;
    command.args(["show", "--pretty=", "--name-only", c_hash]);
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

pub async fn extract_details_parallel(file_path: String) -> Vec<AuthorDetailsV2> {
    // For now - this is not parallelized, TODO: @krshrimali.
    // First get all the commit hashes that ever touched the given file path.
    let commit_hashes = git_command_algo::get_all_commits_for_file(file_path.clone());
    let mut map: HashMap<u32, Vec<diff_v2::LineDetail>> = HashMap::new();
    for commit_hash in commit_hashes.iter() {
        diff_v2::extract_commit_hashes(commit_hash, &mut map, file_path.as_str());
    }
    // Map has populated "relevant commit hashes" for each line.
    // Now use those commit hashes to find the most relevant files for each line.
    let mut author_details_vec: Vec<AuthorDetailsV2> = Vec::new();
    for (line_number, line_detail) in map.iter() {
        // author_full_name is a TODO.
        let author_details = AuthorDetailsV2 {
            origin_file_path: file_path.clone(),
            line_number: *line_number as usize,
            commit_hashes: line_detail[0].commit_hashes.clone(),
            author_full_name: Vec::new(),
        };
        author_details_vec.push(author_details);
    }
    author_details_vec
}

pub fn get_all_commits_for_file(file_path: String) -> Vec<String> {
    // git log --pretty=format:"%h" --reverse -- file_path
    let mut command = Command::new("git");
    command.args([
        "log",
        // "--no-merges",
        "--pretty=format:%h",
        "--reverse",
        "--",
        file_path.as_str(),
    ]);
    let output = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap();
    let stdout_buf = String::from_utf8(output.stdout).unwrap();
    let mut commits: Vec<String> = Vec::new();
    for line in stdout_buf.lines() {
        commits.push(line.to_string());
    }
    commits
}

pub fn get_commit_descriptions(commit_hashes: Vec<String>) -> Vec<Vec<String>> {
    let mut output = Vec::new();
    let mut visited_commits = HashSet::new();

    for commit_hash in commit_hashes.iter() {
        if visited_commits.contains(commit_hash) {
            continue;
        }
        // First get the commit title:
        let mut commit_title = String::new();
        let mut commit_description = String::new();
        if let Ok(output) = Command::new("git").args(["show", "-s", "--format=%s", commit_hash]).output() {
            if output.status.success() {
                if let Ok(title) = String::from_utf8(output.stdout) {
                    commit_title = title.trim().to_string();
                }
            }
        }
        if let Ok(output) = Command::new("git").args(["show", "-s", "--format=%b", commit_hash]).output() {
            if output.status.success() {
                visited_commits.insert(commit_hash.clone());
                if let Ok(desc) = String::from_utf8(output.stdout) {
                    commit_description = desc.trim().to_string();
                }
            }
        }
        output.push(vec![commit_title, commit_description]);
    }
    output
}
