use similar::{ChangeTag, TextDiff};
use std::collections::{HashMap, VecDeque};
use std::process::Command;
use strsim::levenshtein;

#[derive(Debug, Clone)]
struct LineTracker {
    content: String,
    history: Vec<String>,
    origin: Option<usize>,
}

fn normalized_similarity(a: &str, b: &str) -> f64 {
    let dist = levenshtein(a, b) as f64;
    let max_len = a.len().max(b.len()) as f64;
    if max_len == 0.0 {
        1.0
    } else {
        1.0 - dist / max_len
    }
}

fn get_commits(file: &str) -> Vec<String> {
    let output = Command::new("git")
        .args(["log", "--reverse", "--format=%H", "--", file])
        .output()
        .expect("Failed to run git log");

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

fn get_file_at_commit(file: &str, commit: &str) -> Vec<String> {
    let spec = format!("{}:{}", commit, file);
    let output = Command::new("git")
        .args(["show", &spec])
        .output()
        .unwrap_or_else(|_| panic!("Failed to git show {}", commit));

    if output.status.success() {
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.to_string())
            .collect()
    } else {
        vec![]
    }
}

pub fn track_line_movement(file: &str) -> HashMap<usize, Vec<String>> {
    let commits = get_commits(file);
    println!("Total commits to process: {}", commits.len());
    let mut prev_content: Vec<String> = Vec::new();
    let mut prev_lines: Vec<LineTracker> = Vec::new();

    for (i, commit) in commits.iter().enumerate() {
        println!(
            "Processing commit {} / {}: {}",
            i + 1,
            commits.len(),
            commit
        );
        let new_content = get_file_at_commit(file, commit);
        let old: Vec<&str> = prev_content.iter().map(AsRef::as_ref).collect();
        let new: Vec<&str> = new_content.iter().map(AsRef::as_ref).collect();
        let diff = TextDiff::from_slices(&old, &new);

        let mut new_lines: Vec<LineTracker> = Vec::new();
        let mut old_index = 0;
        let mut delete_buffer: VecDeque<(usize, String, LineTracker)> = VecDeque::new();

        for change in diff.iter_all_changes() {
            println!("Change: {:?}", change.clone());
            match change.tag() {
                ChangeTag::Delete => {
                    if let Some(old_line) = prev_lines.get(old_index) {
                        delete_buffer.push_back((old_index, change.to_string(), old_line.clone()));
                    }
                    old_index += 1;
                }
                ChangeTag::Insert => {
                    let inserted = change.to_string();
                    let mut matched = false;
                    for (idx, (old_idx, deleted, tracker)) in delete_buffer.iter().enumerate() {
                        println!("Similarity b/w {} and {}: {}", deleted, inserted, normalized_similarity(deleted, &inserted));
                        if normalized_similarity(&deleted, &inserted) >= 0.7 {
                            let mut updated = tracker.clone();
                            updated.content = inserted.clone();
                            if !updated.history.contains(commit) {
                                updated.history.push(commit.clone());
                            }
                            new_lines.push(updated);
                            delete_buffer.remove(idx);
                            matched = true;
                            break;
                        }
                    }
                    if !matched {
                        new_lines.push(LineTracker {
                            content: inserted.clone(),
                            history: vec![commit.clone()],
                            origin: None,
                        });
                    }
                }
                ChangeTag::Equal => {
                    if let Some(old_line) = prev_lines.get(old_index) {
                        let mut updated = old_line.clone();
                        if !updated.history.contains(commit) {
                            updated.history.push(commit.clone());
                        }
                        new_lines.push(updated);
                    } else {
                        new_lines.push(LineTracker {
                            content: change.to_string(),
                            history: vec![commit.clone()],
                            origin: Some(old_index + 1),
                        });
                    }
                    old_index += 1;
                }
            }
        }

        prev_content = new_content;
        prev_lines = new_lines;
    }

    let mut result: HashMap<usize, Vec<String>> = HashMap::new();
    for (i, line) in prev_lines.iter().enumerate() {
        let mut commits = line.history.clone();
        commits.sort();
        commits.dedup();
        result.insert(i + 1, commits);
    }
    result
}
