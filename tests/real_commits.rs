// Testing on real git commits (From context-pilot-rs itself)
use contextpilot::contextgpt_structs::AuthorDetailsV2;
use contextpilot::diff_v2::{LineDetail, extract_commit_hashes};
use std::collections::HashMap;

#[cfg(test)]
mod tests_real_commits {
    use std::process::{Command, Stdio};

    use super::*;

    #[tokio::test]
    async fn test_real_commits() {
        let all_commits = "tests/real_commits.txt".to_string();
        // In format: "commit1,commit2"
        let all_commits = std::fs::read_to_string(all_commits).expect("Failed to read file");
        let all_commits: Vec<&str> = all_commits.trim().split(',').collect();
        assert!(!all_commits.is_empty(), "No commits found in the file");
        // Use the last commit hash as the one to check blame at.
        let last_commit_hash = all_commits.last().unwrap();
        let mut map: HashMap<u32, Vec<LineDetail>> = HashMap::new();
        for commit_hash in all_commits.iter() {
            extract_commit_hashes(
                commit_hash,
                &mut map,
                "src/main.rs"
                    .to_string()
                    .as_str(),
            );
        }

        // let mut author_details_vec: Vec<AuthorDetailsV2> = Vec::new();
        let mut auth_details_map: HashMap<u32, AuthorDetailsV2> = HashMap::new();
        let file_path = String::from("src/main.rs");
        // Sort the map keys:
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
            // author_details_vec.push(author_details);
        }

        let mut total_count = 0;
        let mut failed_count = 0;
        // Find accuracy of the indexing:
        // Accuracy is defined as, as the output for each line of code - the last commit should always
        // be coming from git blame.
        for (line_number, line_detail) in map.iter() {
            if line_detail.get(0).unwrap().content.is_empty() {
                continue;
            }
            // Find the git blame from the line_number:
            let mut command = Command::new("git");
            command.args([
                "blame",
                last_commit_hash, // Use the last commit hash from the file.
                // "dde6f56",
                // "80e157a",
                "-L",
                &format!("{},{}", line_number, line_number),
                "--abbrev=7",
                "--",
                file_path.as_str(),
            ]);
            let output = command
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .unwrap();
            let stdout_buf = String::from_utf8(output.stdout).unwrap();
            // Extract commit hash from: c5bca082 (Kushashwa Ravi Shrimali 2023-10-21 16:52:43 +0530 1) mod algo_loc;
            let mut commit_hash = String::new();
            if let Some(first_line) = stdout_buf.lines().next() {
                // Split by space and take the first part as commit hash.
                let parts: Vec<&str> = first_line.split_whitespace().collect();
                if !parts.is_empty() {
                    commit_hash = parts[0].to_string();
                }
            } else {
                continue;
            }
            // Check if commit hash == author_details_vec
            let author_detail = auth_details_map.get(line_number);
            if let Some(author_detail) = author_detail {
                // If the commit hash is not already in the commit_hashes, add it.
                if commit_hash.starts_with("^") {
                    // Make sure this is included as well...
                    let commit_hash = commit_hash.strip_prefix("^").unwrap();
                    if author_detail
                        .commit_hashes
                        .contains(&commit_hash.to_string())
                    {
                        total_count += 1;
                    } else {
                        failed_count += 1;
                    }
                } else {
                    // Just take 7 first chars:
                    let commit_hash = &commit_hash[..7];
                    // println!("Searching for commit hash: {}", commit_hash);
                    if author_detail
                        .commit_hashes
                        .contains(&commit_hash.to_string())
                    {
                        total_count += 1;
                    } else {
                        failed_count += 1;
                        // println!(
                        //     "Commit hash {} not found in author details for line {}",
                        //     commit_hash, line_number
                        // );
                        // println!("Author details: {:?}", author_detail.commit_hashes);
                    }
                }
            }
        }

        // Print the whole map with line number + commit hashes:
        // First sort the map according to the line numbers:
        let mut sorted_keys: Vec<u32> = map.keys().copied().collect();
        sorted_keys.sort();
        let mut sorted_map: HashMap<u32, Vec<LineDetail>> = HashMap::new();
        for key in sorted_keys.clone() {
            if let Some(value) = map.get(&key) {
                sorted_map.insert(key, value.clone());
            }
        }
        // Now print the sorted map:
        // println!("Sorted map:");
        for line_number in sorted_keys.iter() {
            let line_detail = sorted_map.get(line_number).unwrap();
            println!(
                "Line {}: {:?}",
                line_number,
                line_detail
            );
        }

        println!(
            "Accuracy for file {} : {}/{}",
            file_path.clone(),
            total_count,
            total_count + failed_count
        );
        // At the end, for now, just print the map:
        println!("Length of map: {}", map.len());
        assert!(map.len() == 113, "Map should not be empty");
    }
}
