use std::collections::HashMap;
use std::process::Command;
use crate::git_command_algo;
use crate::diff_v2::LineDetail;
use crate::git_command_algo::extract_details_parallel;

fn prep(file_path: String) {
    git_command_algo::get_all_commits_for_file(file_path);
}

pub async fn accuracy() {
    // Get files in "src" folder:
    let file_path: String = String::from("src/main.rs");
    prep(file_path.clone());
    // Get all commits for the file path:
    let auth_impl = extract_details_parallel(file_path.clone()).await;
    println!("Total length of the map: {:?}", auth_impl.len());
    let mut good = 0;
    let mut bad = 0;
    for (idx, auth_detail) in auth_impl.iter() {
        let cmt = auth_detail.commit_hashes.last().unwrap();
        // Perform git blame on the "idx" in the file_path above:
        // Find the git blame from the line_number:
        let mut command = Command::new("git");
        command.args([
            "blame",
            "-L",
            &format!("{},{}", idx, idx),
            "--abbrev=7",
            "--",
            file_path.clone().as_str(),
        ]);
        let output = command.output().expect("failed to execute process");
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Parse commit_hash from the output:
        // Get the first word in stdout:
        let mut words = stdout.split_whitespace();
        // Ensure words isn't empty:
        let word = words.next();
        if word.is_none() {
            println!("Empty stdout");
            bad += 1;
        } else {
            let mut commit_hash = word.unwrap();
            // Get first 7 characters of commit_hash:
            commit_hash = &commit_hash[0..7];
            if commit_hash == cmt {
                good += 1;
            } else {
                bad += 1;
            }
        }
    }
    println!("Good: {}, Bad: {}", good, bad);
}