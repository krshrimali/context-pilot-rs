use crate::{contextgpt_structs::AuthorDetailsV2, diff_v2};

use crate::git_command_algo;
use futures::stream::{FuturesUnordered, StreamExt};
use rayon::prelude::*;
use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    process::{Command, Stdio},
};
use tokio::sync::Semaphore;

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

pub fn analyze_file_history(filename: &str) -> HashMap<usize, Vec<CommitInfo>> {
    let mut tracks = read_current_file(filename);
    let commits = parse_git_log(filename);
    update_tracks(&mut tracks, commits);
    let blame_map = fetch_blame(filename);
    fill_missing_with_blame(&mut tracks, blame_map);

    let mut result = HashMap::new();
    for (i, track) in tracks.iter().enumerate() {
        result.insert((i + 1) as usize, track.commits.clone());
    }
    result
}

fn analyze_file_history_safe(filename: &str) -> Result<HashMap<usize, Vec<CommitInfo>>, ()> {
    Ok(analyze_file_history(filename)) // you can catch errors inside if needed
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

// pub async fn extract_details_parallel(file_path: String) -> Vec<AuthorDetailsV2> {
//     let semaphore = Arc::new(Semaphore::new(8)); // limit to 8 concurrent file processes
//     let mut tasks = FuturesUnordered::new();
//
//     let permit = semaphore.clone().acquire_owned().await.unwrap();
//     let file_path_clone = file_path.clone();
//
//     tasks.push(tokio::spawn(async move {
//         let _permit = permit; // keep permit alive
//         match analyze_file_history_safe(file_path_clone.as_str()) {
//             Ok(output) => {
//                 let mut author_details_vec = Vec::new();
//                 for (line_number, commit_info) in output.iter() {
//                     if commit_info.is_empty() {
//                         continue;
//                     }
//                     let mut author_details = AuthorDetailsV2 {
//                         origin_file_path: file_path.clone(),
//                         line_number: *line_number,
//                         commit_hashes: Vec::new(),
//                         author_full_name: Vec::new(),
//                     };
//                     for commit in commit_info {
//                         author_details.commit_hashes.push(commit.hash.clone());
//                         author_details.author_full_name.push(commit.message.clone());
//                     }
//                     author_details_vec.push(author_details);
//                 }
//                 author_details_vec
//             }
//             Err(_) => Vec::new(), // If analyze fails, return empty
//         }
//     }));
//
//     let mut results = Vec::new();
//     while let Some(task_result) = tasks.next().await {
//         match task_result {
//             Ok(author_details) => results.extend(author_details),
//             Err(e) => eprintln!("Task failed: {:?}", e),
//         }
//     }
//
//     results
// }

pub fn extract_details(file_path: String) -> Vec<AuthorDetailsV2> {
    let output = analyze_file_history(file_path.as_str());
    let mut author_details_vec: Vec<AuthorDetailsV2> = Vec::new();
    for (line_number, commit_info) in output.iter() {
        if commit_info.is_empty() {
            continue;
        }
        let mut author_details = AuthorDetailsV2 {
            origin_file_path: file_path.clone(),
            line_number: *line_number,
            commit_hashes: Vec::new(),
            author_full_name: Vec::new(),
        };
        for commit in commit_info {
            author_details.commit_hashes.push(commit.hash.clone());
            author_details.author_full_name.push(commit.message.clone());
        }
        author_details_vec.push(author_details);
    }
    author_details_vec
}

#[derive(Debug, Clone)]
struct CommitInfo {
    hash: String,
    message: String,
}

#[derive(Debug)]
struct LineTrack {
    current_line_number: usize, // 1-indexed
    active: bool,
    commits: Vec<CommitInfo>,
}

fn read_current_file(filename: &str) -> Vec<LineTrack> {
    let file = File::open(filename).expect("Cannot open file");
    let reader = BufReader::new(file);

    reader
        .lines()
        .enumerate()
        .filter_map(|(idx, line)| {
            line.ok().map(|_| LineTrack {
                current_line_number: idx + 1,
                active: true,
                commits: Vec::new(),
            })
        })
        .collect()
}

fn parse_git_log(filename: &str) -> Vec<(CommitInfo, Vec<String>)> {
    let output = Command::new("git")
        .args(["log", "--patch", "--unified=0", filename])
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to run git log")
        .stdout
        .expect("Failed to capture stdout");

    let reader = BufReader::new(output);

    let re_commit = Regex::new(r"^commit ([a-f0-9]+)").unwrap();
    let re_message = Regex::new(r"^\s{4}(.*)").unwrap();
    let mut commits = Vec::new();

    let mut current_commit_hash = String::new();
    let mut current_message = String::new();
    let mut current_patch = Vec::new();
    let mut reading_message = false;

    for line in reader.lines() {
        let line = line.expect("Failed to read line");

        if let Some(caps) = re_commit.captures(&line) {
            if !current_commit_hash.is_empty() {
                commits.push((
                    CommitInfo {
                        hash: current_commit_hash[..7].to_string(),
                        message: current_message.clone(),
                    },
                    current_patch.clone(),
                ));
            }
            current_commit_hash = caps[1].to_string();
            current_message.clear();
            current_patch.clear();
            reading_message = true;
        } else if reading_message {
            if let Some(caps) = re_message.captures(&line) {
                current_message = caps[1].to_string();
                reading_message = false;
            }
        } else if line.starts_with("@@") || line.starts_with('+') || line.starts_with('-') {
            current_patch.push(line);
        }
    }

    if !current_commit_hash.is_empty() {
        commits.push((
            CommitInfo {
                hash: current_commit_hash[..7].to_string(),
                message: current_message,
            },
            current_patch,
        ));
    }

    commits
}

fn parse_hunk_header(header: &str) -> Option<(usize, usize, usize, usize)> {
    let re_hunk = Regex::new(r"^@@ -(\d+),?(\d+)? \+(\d+),?(\d+)? @@").unwrap();
    re_hunk.captures(header).map(|caps| {
        let old_start = caps[1].parse().unwrap_or(1);
        let old_len = caps.get(2).map_or(1, |m| m.as_str().parse().unwrap_or(1));
        let new_start = caps[3].parse().unwrap_or(1);
        let new_len = caps.get(4).map_or(1, |m| m.as_str().parse().unwrap_or(1));
        (old_start, old_len, new_start, new_len)
    })
}

fn update_tracks(tracks: &mut [LineTrack], commits: Vec<(CommitInfo, Vec<String>)>) {
    for (commit_info, patch) in commits {
        let mut line_map: HashMap<usize, (char, String)> = HashMap::new();
        let mut cur_old_line = 0;
        let mut cur_new_line = 0;

        for line in patch {
            if line.starts_with("@@") {
                if let Some((old_start, _old_len, new_start, _new_len)) = parse_hunk_header(&line) {
                    cur_old_line = old_start;
                    cur_new_line = new_start;
                }
            } else if line.starts_with('+') && !line.starts_with("+++") {
                line_map.insert(cur_new_line, ('+', line.clone()));
                cur_new_line += 1;
            } else if line.starts_with('-') && !line.starts_with("---") {
                line_map.insert(cur_old_line, ('-', line.clone()));
                cur_old_line += 1;
            } else {
                cur_old_line += 1;
                cur_new_line += 1;
            }
        }

        for track in tracks.iter_mut().filter(|t| t.active) {
            let pos = track.current_line_number;

            if let Some((op, _line)) = line_map.get(&pos) {
                match op {
                    '+' => {
                        track.commits.push(commit_info.clone());
                        track.active = false;
                    }
                    '-' => {
                        track.commits.push(commit_info.clone());
                    }
                    _ => {}
                }
            }
        }

        let mut inserts: Vec<usize> = line_map
            .iter()
            .filter_map(|(k, (op, _))| if *op == '+' { Some(*k) } else { None })
            .collect();
        let mut deletes: Vec<usize> = line_map
            .iter()
            .filter_map(|(k, (op, _))| if *op == '-' { Some(*k) } else { None })
            .collect();

        inserts.sort_unstable();
        deletes.sort_unstable();

        for insert_pos in inserts {
            for track in tracks.iter_mut().filter(|t| t.active) {
                if track.current_line_number >= insert_pos {
                    track.current_line_number += 1;
                }
            }
        }

        for delete_pos in deletes {
            for track in tracks.iter_mut().filter(|t| t.active) {
                if track.current_line_number > delete_pos {
                    track.current_line_number -= 1;
                }
            }
        }
    }
}

fn fetch_blame(filename: &str) -> HashMap<usize, CommitInfo> {
    use regex::Regex;
    use std::collections::HashMap;
    use std::process::Command;

    let output = Command::new("git")
        .args(["blame", "--line-porcelain", filename])
        .output()
        .expect("Failed to run git blame");

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut lines = output_str.lines().peekable();

    let mut blame_map = HashMap::new();
    let mut current_hash = String::new();
    let mut current_summary = String::new();
    let mut current_line_number = 1;

    while let Some(line) = lines.next() {
        if let Some(caps) = Regex::new(r"^([a-f0-9]{40}) ").unwrap().captures(line) {
            // New commit block starts
            current_hash = caps[1][..7].to_string(); // Short hash
            current_summary.clear();
        } else if line.starts_with("summary ") {
            current_summary = line[8..].to_string();
        } else if line.starts_with(char::is_alphabetic) {
            // metadata line (author, committer, filename, etc.) — skip
            continue;
        } else {
            // Real content line detected
            blame_map.insert(
                current_line_number,
                CommitInfo {
                    hash: current_hash.clone(),
                    message: current_summary.clone(),
                },
            );
            current_line_number += 1;
        }
    }

    blame_map
}

fn fill_missing_with_blame(tracks: &mut [LineTrack], blame_map: HashMap<usize, CommitInfo>) {
    for (i, track) in tracks.iter_mut().enumerate() {
        if track.commits.is_empty() {
            if let Some(commit_info) = blame_map.get(&(i + 1)) {
                track.commits.push(commit_info.clone());
            }
        }
    }
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
