use ignore::Walk;
use ignore::gitignore::GitignoreBuilder;

use crate::{contextgpt_structs::AuthorDetailsV2, diff_v2};

use crate::git_command_algo;
use std::collections::{HashMap, HashSet};
use std::process::{Command, Stdio};

pub fn print_all_valid_directories(
    workspace_dir: String,
    gitignore_file_name: Option<String>,
) {
    // Prints all the valid files to stdout - used by plugins
    // optionally to get files that are to be indexed.
    let gitignore_file_name = gitignore_file_name.unwrap_or(String::from(".gitignore"));
    let mut gitignore_builder = GitignoreBuilder::new(workspace_dir.clone());
    gitignore_builder.add(gitignore_file_name);
    let gitignore = gitignore_builder.build().expect("Failed");
    let mut all_paths: Vec<String> = vec![];
    // Iterate through all the files in the workspace_dir:
    for walk_entry in Walk::new(workspace_dir.clone()) {
        match walk_entry {
            Ok(entry) => {
                let path = entry.path();
                if path.is_file() {
                    continue;
                    // // Check if the file is ignored
                    // if gitignore.matched(path, false).is_ignore() {
                    //     continue;
                    // }
                    // // Print the file path -- it's valid!
                } else {
                    // Check if the whole dir is ignored:
                    if gitignore.matched(path, true).is_ignore() {
                        // Skip the directory.
                        continue;
                    }
                    // Print the relative path only:
                    let rel_path = path.strip_prefix(workspace_dir.clone());
                    if rel_path.is_ok() {
                        let relative_path = rel_path.clone().unwrap();
                        if !relative_path.to_path_buf().to_string_lossy().is_empty() {
                            all_paths.push(relative_path.display().to_string());
                        }
                    }
                }
            }
            Err(err) => {
                eprintln!("Error: {}", err);
            }
        }
    }
    println!("{:?}", all_paths);
}

pub fn print_all_valid_files(workspace_dir: String, gitignore_file_name: Option<String>) -> () {
    // Prints all the valid files to stdout - used by plugins
    // optionally to get files that are to be indexed.
    let gitignore_file_name = gitignore_file_name.unwrap_or(String::from(".gitignore"));
    let mut gitignore_builder = GitignoreBuilder::new(workspace_dir.clone());
    gitignore_builder.add(gitignore_file_name);
    let gitignore = gitignore_builder.build().expect("Failed");
    // Iterate through all the files in the workspace_dir:
    for walk_entry in Walk::new(workspace_dir.clone()) {
        match walk_entry {
            Ok(entry) => {
                let path = entry.path();
                if path.is_file() {
                    // Check if the file is ignored
                    if gitignore.matched(path, false).is_ignore() {
                        continue;
                    }
                    // Print the file path -- it's valid!
                    println!("{}", path.display());
                } else {
                    // Check if the whole dir is ignored:
                    if gitignore.matched(path, true).is_ignore() {
                        // Skip the directory.
                        continue;
                    }
                }
            }
            Err(err) => {
                eprintln!("Error: {}", err);
            }
        }
    }
}

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


pub async fn index_some_commits(
    origin_file_path: String,
    commits_to_index: Vec<String>,
) -> HashMap<u32, AuthorDetailsV2> {
    // For now - this is not parallelized, TODO: @krshrimali.
    // First get all the commit hashes that ever touched the given file path.
    let mut map: HashMap<u32, Vec<diff_v2::LineDetail>> = HashMap::new();
    let mut parent_commit_hash: String = String::from("");
    for commit_hash in commits_to_index.iter() {
        diff_v2::extract_commit_hashes(&parent_commit_hash, commit_hash, &mut map, origin_file_path.as_str());
        parent_commit_hash = commit_hash.clone();
    }
    // Map has populated "relevant commit hashes" for each line.
    // Now use those commit hashes to find the most relevant files for each line.
    let mut auth_details_map: HashMap<u32, AuthorDetailsV2> = HashMap::new();
    let mut sorted_keys: Vec<u32> = map.keys().copied().collect();
    sorted_keys.sort();
    for line_number in sorted_keys.iter() {
        let line_detail = map.get(line_number).unwrap();
        // author_full_name is a TODO.
        let author_details = AuthorDetailsV2 {
            origin_file_path: origin_file_path.clone(),
            line_number: *line_number as usize,
            commit_hashes: line_detail[0].commit_hashes.clone(),
            author_full_name: Vec::new(),
        };
        auth_details_map.insert(*line_number, author_details.clone());
    }
    auth_details_map
}

pub async fn extract_details_parallel(file_path: String) -> HashMap<u32, AuthorDetailsV2> {
    // For now - this is not parallelized, TODO: @krshrimali.
    // First get all the commit hashes that ever touched the given file path.
    let commit_hashes = git_command_algo::get_all_commits_for_file(file_path.clone());
    let mut map: HashMap<u32, Vec<diff_v2::LineDetail>> = HashMap::new();
    let mut parent_commit_hash: String = String::from("");
    for commit_hash in commit_hashes.iter() {
        diff_v2::extract_commit_hashes(&parent_commit_hash, commit_hash, &mut map, file_path.as_str());
        parent_commit_hash = commit_hash.clone();
    }
    // Map has populated "relevant commit hashes" for each line.
    // Now use those commit hashes to find the most relevant files for each line.
    let mut auth_details_map: HashMap<u32, AuthorDetailsV2> = HashMap::new();
    let mut sorted_keys: Vec<u32> = map.keys().copied().collect();
    sorted_keys.sort();
    for line_number in sorted_keys.iter() {
        let line_detail = map.get(line_number).unwrap();
        // author_full_name is a TODO.
        let author_details = AuthorDetailsV2 {
            origin_file_path: file_path.clone(),
            line_number: *line_number as usize,
            commit_hashes: line_detail[0].commit_hashes.clone(),
            author_full_name: Vec::new(),
        };
        auth_details_map.insert(*line_number, author_details.clone());
    }
    auth_details_map
}

pub fn get_all_commits_for_file(file_path: String) -> Vec<String> {
    let mut command = Command::new("git");
    command.args([
        "log",
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
    // Ensure commits contains git blame output as well for each line.
    // This is to ensure that we have the commit hashes in the order they were made.
    if commits.is_empty() {
        // If no commits found, return an empty vector.
        return commits;
    }
    // Add the last commit hash as well, which is the current state of the file.
    let mut command = Command::new("git");
    command.args(["log", "--pretty=format:%h", "--", file_path.as_str()]);
    let output = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap();
    let stdout_buf = String::from_utf8(output.stdout).unwrap();
    // For each line number - create another hashmap.
    let mut last_commit_map: HashMap<usize, String> = HashMap::new();
    for (idx, line) in stdout_buf.lines().enumerate() {
        let commit_hash = line.to_string();
        last_commit_map.insert(idx, commit_hash.clone());
    }
    // Now iterate through last_commit_map and check if it is in commits.
    for (_, commit_hash) in last_commit_map.iter() {
        if !commits.contains(commit_hash) {
            commits.push(commit_hash.clone());
        }
    }
    commits
}

fn get_commit_base_url() -> Option<String> {
    if let Ok(output) = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()
    {
        if output.status.success() {
            if let Ok(url) = String::from_utf8(output.stdout) {
                let url = url.trim();
                // Handle GitHub URLs (both HTTPS and SSH)
                if url.starts_with("git@github.com:") {
                    let path = url.strip_prefix("git@github.com:").unwrap();
                    // Optionally strip ".git" if present
                    let path = path.strip_suffix(".git").unwrap_or(path);
                    return Some(format!("https://github.com/{}/commit/", path));
                } else if url.starts_with("https://github.com/") {
                    let path = url.strip_prefix("https://github.com/").unwrap();
                    // Optionally strip ".git" if present
                    let path = path.strip_suffix(".git").unwrap_or(path);
                    return Some(format!("https://github.com/{}/commit/", path));
                }
            }
        }
    }
    None
}

pub fn get_commit_descriptions(commit_hashes: Vec<String>) -> Vec<Vec<String>> {
    let mut output_vec = Vec::new();
    let mut visited_commits = HashSet::new();

    let base_url = get_commit_base_url();

    for commit_hash in commit_hashes.iter() {
        if visited_commits.contains(commit_hash) {
            continue;
        }

        if let Ok(output) = Command::new("git")
            .args([
                "show",
                "-s",
                "--format=%s%n%b%n--AUTHOR--%n%an%n--DATE--%n%cd",
                "--date=local",
                commit_hash,
            ])
            .output()
        {
            if output.status.success() {
                visited_commits.insert(commit_hash.clone());
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    let sections: Vec<&str> = output_str.split("\n--AUTHOR--\n").collect();
                    if sections.len() == 2 {
                        let message = sections[0].trim();
                        let mut lines = message.lines();
                        let commit_title = lines.next().unwrap_or("").trim().to_string();
                        let commit_description =
                            lines.collect::<Vec<_>>().join("\n").trim().to_string();

                        let parts: Vec<&str> = sections[1].split("\n--DATE--\n").collect();
                        if parts.len() == 2 {
                            let author_name = parts[0].trim().to_string();
                            let commit_datetime = parts[1].trim().to_string();

                            let commit_url = base_url
                                .as_ref()
                                .map(|url| format!("{}{}", url, commit_hash))
                                .unwrap_or_else(|| "".to_string());

                            output_vec.push(vec![
                                commit_title,
                                commit_description,
                                author_name,
                                commit_datetime,
                                commit_url,
                            ]);
                        }
                    }
                }
            }
        }
    }
    output_vec
}

pub fn get_latest_commit(file_path: &String) -> Option<String> {
    // Get the latest commit hash for the given file path.
    let mut command = Command::new("git");
    command.args(["log", "-1", "--pretty=format:%h", "--", file_path.as_str()]);
    let output = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap();
    if output.status.success() {
        if let Ok(commit_hash) = String::from_utf8(output.stdout) {
            let commit_hash = commit_hash.trim().to_string();
            if !commit_hash.is_empty() {
                return Some(commit_hash);
            }
        }
    }
    None
}

pub fn get_commits_after(last_indexed_commit: String) -> Vec<String> {
    let mut command = Command::new("git");
    command.args(["rev-list", &last_indexed_commit, "..", "HEAD"]);

    let output = match command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
    {
        Ok(output) => output,
        Err(e) => {
            eprintln!("❌ Failed to execute git command: {}", e);
            return Vec::new();
        }
    };

    if output.status.success() {
        match String::from_utf8(output.stdout) {
            Ok(stdout_buf) => {
                let commits: Vec<String> = stdout_buf.lines()
                    .filter(|line| !line.trim().is_empty())
                    .map(|s| s.to_string())
                    .collect();
                return commits;
            }
            Err(e) => {
                eprintln!("❌ Failed to parse git output as UTF-8: {}", e);
                return Vec::new();
            }
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("❌ Git command failed: {}", stderr.trim());
        if stderr.contains("bad revision") || stderr.contains("unknown revision") {
            eprintln!("⚠ Invalid commit hash: {}", last_indexed_commit);
        }
    }

    Vec::new()
}