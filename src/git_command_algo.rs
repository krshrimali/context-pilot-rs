use ignore::Walk;
use ignore::gitignore::GitignoreBuilder;

use crate::{contextgpt_structs::AuthorDetailsV2, diff_v2};

use crate::git_command_algo;
use std::collections::{HashMap, HashSet};
use std::process::{Command, Stdio};

#[derive(Debug, Clone, PartialEq)]
pub struct FileRename {
    pub old_path: String,
    pub new_path: String,
    pub commit_hash: String,
    pub similarity: u32,
}

pub fn print_all_valid_directories(workspace_dir: String, gitignore_file_name: Option<String>) {
    // Prints all the valid files to stdout - used by plugins
    // optionally to get files that are to be indexed.
    // if gitignore_file_name.is_none() {
    //     println!("None.");
    //     return;
    // }
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
                    // println!("{}", path.display());
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
                eprintln!("Error: {err}");
            }
        }
    }
    println!("{all_paths:?}");
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
    let mut current_file_path = origin_file_path.clone();

    for commit_hash in commits_to_index.iter() {
        // Check if this commit contains a rename for our current file
        if let Some(rename) = detect_rename_in_commit(commit_hash, &current_file_path) {
            // Process the diff with the new path
            diff_v2::extract_commit_hashes(
                &parent_commit_hash,
                commit_hash,
                &mut map,
                current_file_path.as_str(),
            );
            // Update current path to the old path for subsequent commits
            current_file_path = rename.old_path.clone();
        } else {
            // Normal commit, no rename
            diff_v2::extract_commit_hashes(
                &parent_commit_hash,
                commit_hash,
                &mut map,
                current_file_path.as_str(),
            );
        }
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
    let mut current_file_path = file_path.clone();

    for commit_hash in commit_hashes.iter() {
        // Check if this commit contains a rename for our current file
        if let Some(rename) = detect_rename_in_commit(commit_hash, &current_file_path) {
            // Process the diff with the new path
            diff_v2::extract_commit_hashes(
                &parent_commit_hash,
                commit_hash,
                &mut map,
                current_file_path.as_str(),
            );
            // Update current path to the old path for subsequent commits
            current_file_path = rename.old_path.clone();
        } else {
            // Normal commit, no rename
            diff_v2::extract_commit_hashes(
                &parent_commit_hash,
                commit_hash,
                &mut map,
                current_file_path.as_str(),
            );
        }
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
    // FIXME: @krshrimali - Remove this once proper testing is done.
    // let mut total_count = 0;
    // let mut failed_count = 0;
    // // Find accuracy of the indexing:
    // // Accuracy is defined as, as the output for each line of code - the last commit should always
    // // be coming from git blame.
    // for (line_number, line_detail) in map.iter() {
    //     if line_detail.get(0).unwrap().content.is_empty() {
    //         continue;
    //     }
    //     // Find the git blame from the line_number:
    //     let mut command = Command::new("git");
    //     command.args([
    //         "blame",
    //         "-L",
    //         &format!("{},{}", line_number, line_number),
    //         "--abbrev=7",
    //         "--",
    //         file_path.as_str(),
    //     ]);
    //     let output = command
    //         .stdout(Stdio::piped())
    //         .stderr(Stdio::piped())
    //         .output()
    //         .unwrap();
    //     let stdout_buf = String::from_utf8(output.stdout).unwrap();
    //     // Extract commit hash from: c5bca082 (Kushashwa Ravi Shrimali 2023-10-21 16:52:43 +0530 1) mod algo_loc;
    //     let mut commit_hash = String::new();
    //     if let Some(first_line) = stdout_buf.lines().next() {
    //         // Split by space and take the first part as commit hash.
    //         let parts: Vec<&str> = first_line.split_whitespace().collect();
    //         if !parts.is_empty() {
    //             commit_hash = parts[0].to_string();
    //         }
    //     }
    //     // Check if commit hash == author_details_vec
    //     let author_detail = auth_details_map.get(line_number);
    //     if let Some(author_detail) = author_detail {
    //         // If the commit hash is not already in the commit_hashes, add it.
    //         if commit_hash.starts_with("^") {
    //             // Make sure this is included as well...
    //             let commit_hash = commit_hash.strip_prefix("^").unwrap();
    //             if author_detail
    //                 .commit_hashes
    //                 .contains(&commit_hash.to_string())
    //             {
    //                 if author_detail
    //                     .commit_hashes
    //                     .contains(&commit_hash.to_string())
    //                 {
    //                     total_count += 1;
    //                 } else {
    //                     failed_count += 1;
    //                 }
    //             }
    //         } else {
    //             // Just take 7 first chars:
    //             if commit_hash.len() > 7 {
    //                 commit_hash = commit_hash[..7].to_string();
    //             } else {
    //                 continue;
    //             }
    //             // let commit_hash = &commit_hash[..7];
    //             // println!("Searching for commit hash: {}", commit_hash);
    //             if author_detail
    //                 .commit_hashes
    //                 .contains(&commit_hash.to_string())
    //             {
    //                 total_count += 1;
    //             } else {
    //                 failed_count += 1;
    //                 println!(
    //                     "Commit hash {} not found in author details for line {}",
    //                     commit_hash, line_number
    //                 );
    //                 println!("Author details: {:?}", author_detail.commit_hashes);
    //             }
    //         }
    //     }
    // }
    // println!(
    //     "Accuracy for file {} : {}/{}",
    //     file_path.clone(),
    //     total_count,
    //     total_count + failed_count
    // );
    auth_details_map
}

/// Detects if a file was renamed in a specific commit
/// Returns Some(FileRename) if a rename was detected, None otherwise
pub fn detect_rename_in_commit(commit_hash: &str, file_path: &str) -> Option<FileRename> {
    let mut command = Command::new("git");
    command.args([
        "show",
        "--name-status",
        "-M",
        "--pretty=format:",
        commit_hash,
        "--",
    ]);

    let output = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout_buf = String::from_utf8(output.stdout).ok()?;

    // Parse the output for rename status
    // Format: R<similarity>\told_path\tnew_path
    for line in stdout_buf.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Check if this is a rename line (starts with R followed by a number)
        if line.starts_with('R') {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() != 3 {
                continue;
            }

            let status = parts[0];
            let old_path = parts[1];
            let new_path = parts[2];

            // Check if this rename affects our file
            if new_path == file_path {
                // Extract similarity percentage
                let similarity_str = status.trim_start_matches('R');
                let similarity = similarity_str.parse::<u32>().unwrap_or(100);

                return Some(FileRename {
                    old_path: old_path.to_string(),
                    new_path: new_path.to_string(),
                    commit_hash: commit_hash.to_string(),
                    similarity,
                });
            }
        }
    }

    None
}

/// Gets all file path history for a file, following renames
/// Returns a vector of (file_path, starting_commit) tuples in chronological order
pub fn get_file_rename_history(file_path: String) -> Vec<(String, Option<String>)> {
    let mut history = vec![];
    let mut current_path = file_path.clone();

    // Get all commits using --follow flag
    let mut command = Command::new("git");
    command.args([
        "log",
        "--follow",
        "--name-status",
        "-M",
        "--pretty=format:%h",
        "--",
        &current_path,
    ]);

    let output = match command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
    {
        Ok(output) => output,
        Err(_) => return vec![(file_path, None)],
    };

    if !output.status.success() {
        return vec![(file_path, None)];
    }

    let stdout_buf = String::from_utf8(output.stdout).unwrap_or_default();
    let mut lines = stdout_buf.lines();
    let mut last_rename_commit: Option<String> = None;

    // Track the current path we're following
    history.push((current_path.clone(), None));

    while let Some(line) = lines.next() {
        let line = line.trim();

        // Check if this is a commit hash line
        if !line.is_empty()
            && !line.starts_with('R')
            && !line.starts_with('A')
            && !line.starts_with('M')
            && !line.starts_with('D')
        {
            last_rename_commit = Some(line.to_string());
            continue;
        }

        // Check for rename status
        if line.starts_with('R') {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() == 3 {
                let old_path = parts[1];
                let new_path = parts[2];

                // If we found a rename for our current path
                if new_path == current_path {
                    // Add the old path to history
                    history.push((old_path.to_string(), last_rename_commit.clone()));
                    current_path = old_path.to_string();
                }
            }
        }
    }

    // Reverse to get chronological order (oldest first)
    history.reverse();
    history
}

pub fn get_all_commits_for_file(file_path: String) -> Vec<String> {
    let mut command = Command::new("git");
    command.args([
        "log",
        "--follow", // Follow renames
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
    command.args([
        "log",
        "--follow",
        "--pretty=format:%h",
        "--",
        file_path.as_str(),
    ]);
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
        && output.status.success()
        && let Ok(url) = String::from_utf8(output.stdout)
    {
        let url = url.trim();
        // Handle GitHub URLs (both HTTPS and SSH)
        if url.starts_with("git@github.com:") {
            let path = url.strip_prefix("git@github.com:").unwrap();
            // Optionally strip ".git" if present
            let path = path.strip_suffix(".git").unwrap_or(path);
            return Some(format!("https://github.com/{path}/commit/"));
        } else if url.starts_with("https://github.com/") {
            let path = url.strip_prefix("https://github.com/").unwrap();
            // Optionally strip ".git" if present
            let path = path.strip_suffix(".git").unwrap_or(path);
            return Some(format!("https://github.com/{path}/commit/"));
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
            && output.status.success()
        {
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
                            .map(|url| format!("{url}{commit_hash}"))
                            .unwrap_or_default();

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
    output_vec
}

pub fn get_latest_commit(file_path: &str) -> Option<String> {
    // Get the latest commit hash for the given file path.
    let mut command = Command::new("git");
    command.args(["log", "-1", "--pretty=format:%h", "--", file_path]);
    let output = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap();
    if output.status.success()
        && let Ok(commit_hash) = String::from_utf8(output.stdout)
    {
        let commit_hash = commit_hash.trim().to_string();
        if !commit_hash.is_empty() {
            return Some(commit_hash);
        }
    }
    None
}

pub fn get_commits_after(last_indexed_commit: String) -> Vec<String> {
    // Get all the commits after the last indexed commit.
    // If last_indexed_commit is None, return all commits.
    // If recent_commit is None, return all commits after last_indexed_commit.
    let mut command = Command::new("git");
    command.args(["rev-list", &last_indexed_commit, "..", "HEAD"]);

    let output = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap();

    if output.status.success() {
        let stdout_buf = String::from_utf8(output.stdout).unwrap();
        return stdout_buf.lines().map(|s| s.to_string()).collect();
    }

    Vec::new()
}
