use std::collections::HashMap;
use crate::utils::levenshtein;

/// Main function to extract and track per-line commit history from a commit diff.
pub fn extract_details_with_tracking(
    commit_hash: &str,
    diff_lines: Vec<String>,
    file_path: &str,
    line_commit_map: &mut HashMap<String, HashMap<u32, (String, Vec<String>)>>,
) {
    let mut deleted_lines: Vec<(usize, String)> = Vec::new();
    let mut added_lines: Vec<(usize, String)> = Vec::new();
    let mut matched_additions: Vec<bool> = Vec::new();
    let mut line_num = 0;

    let mut insertion_blocks: Vec<(u32, u32)> = Vec::new();

    let mut i = 0;
    while i < diff_lines.len() {
        let line = &diff_lines[i];

        if line.starts_with("@@") {
            // Before new hunk, finalize unmatched added lines from previous hunk
            for (j, (add_line_no, add_content)) in added_lines.iter().enumerate() {
                if !matched_additions.get(j).unwrap_or(&false) {
                    track_line_commit(file_path, *add_line_no as u32, add_content.clone(), commit_hash.to_string(), line_commit_map);
                }
            }

            deleted_lines.clear();
            added_lines.clear();
            matched_additions.clear();

            // Parse hunk header
            if let Some((_, after)) = line.split_once('+') {
                if let Some((start, count)) = after.split_once(',') {
                    line_num = start.trim().parse::<usize>().unwrap_or(0);

                    // Track insertion blocks only if this hunk has + but no -
                    let mut added_count = count.trim().parse::<usize>().unwrap_or(1);
                    let mut j = i + 1;
                    let mut pure_addition = true;
                    while j < diff_lines.len() && !diff_lines[j].starts_with("@@") {
                        if diff_lines[j].starts_with('-') {
                            pure_addition = false;
                        } else if !diff_lines[j].starts_with('+') {
                            added_count -= 1;
                        }
                        j += 1;
                    }

                    if pure_addition && added_count > 0 {
                        insertion_blocks.push((line_num as u32, added_count as u32));
                    }
                }
            }

            i += 1;
            continue;
        }

        if line.starts_with('-') {
            deleted_lines.push((line_num, line[1..].to_string()));
        } else if line.starts_with('+') {
            added_lines.push((line_num, line[1..].to_string()));
            matched_additions.push(false);
            line_num += 1;
        } else {
            line_num += 1;
        }

        i += 1;
    }

    // After last hunk
    for (j, (add_line_no, add_content)) in added_lines.iter().enumerate() {
        if !matched_additions.get(j).unwrap_or(&false) {
            track_line_commit(file_path, *add_line_no as u32, add_content.clone(), commit_hash.to_string(), line_commit_map);
        }
    }

    // Match deleted to added using Levenshtein
    for (del_line_no, del_content) in &deleted_lines {
        for (i, (add_line_no, add_content)) in added_lines.iter().enumerate() {
            if matched_additions[i] {
                continue;
            }
            let distance = levenshtein(&del_content, &add_content);
            if distance <= 3 {
                track_line_commit(file_path, *add_line_no as u32, add_content.clone(), commit_hash.to_string(), line_commit_map);
                matched_additions[i] = true;
                break;
            }
        }
    }

    // Final shifting for pure insertions
    for (start_line, count) in insertion_blocks {
        shift_lines_after_insert(file_path, start_line, count, line_commit_map);
    }
}

/// Update the commit map for a given file and line.
fn track_line_commit(
    file_path: &str,
    line_number: u32,
    content: String,
    commit_hash: String,
    map: &mut HashMap<String, HashMap<u32, (String, Vec<String>)>>,
) {
    let file_entry = map.entry(file_path.to_string()).or_insert_with(HashMap::new);
    let entry = file_entry.entry(line_number).or_insert((content.clone(), Vec::new()));

    if entry.0 != content {
        entry.0 = content;
        entry.1.push(commit_hash);
    } else if entry.1.last().map(|c| c != &commit_hash).unwrap_or(true) {
        entry.1.push(commit_hash);
    }
}

/// Shift all tracked lines >= `insertion_start` by `added_count` lines.
pub fn shift_lines_after_insert(
    file_path: &str,
    insertion_start: u32,
    added_count: u32,
    map: &mut HashMap<String, HashMap<u32, (String, Vec<String>)>>,
) {
    if let Some(file_map) = map.get_mut(file_path) {
        let mut shifted: HashMap<u32, (String, Vec<String>)> = HashMap::new();
        let mut keys_to_shift: Vec<u32> = file_map
            .keys()
            .cloned()
            .filter(|&k| k >= insertion_start)
            .collect();
        keys_to_shift.sort_unstable_by(|a, b| b.cmp(a)); // Descending

        for key in keys_to_shift {
            if let Some((content, history)) = file_map.remove(&key) {
                shifted.insert(key + added_count, (content, history));
            }
        }

        file_map.extend(shifted);
    }
}
