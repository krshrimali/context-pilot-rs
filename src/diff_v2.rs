use std::{collections::HashMap, str};

#[derive(Debug, Clone)]
struct LineDetail {
    content: String,
    commit_hashes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ChangeType {
    Added,
    Deleted,
    Modified,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LineChange {
    start_line_number: u32,
    change_count: u32,
    change_type: ChangeType,
    changed_content: Vec<String>,
}

impl Default for LineChange {
    fn default() -> Self {
        // Self { start_line_number: Default::default(), change_count: Default::default(), change_type: Default::default(), changed_content: Default::default() }
        Self {
            start_line_number: 0,
            change_count: 0,
            change_type: ChangeType::Added,
            changed_content: vec![],
        }
    }
}

fn is_similar(line_content: &str, added_line_content: &str) -> bool {
    // Check if the line content is similar to the added line content.
    // For now, just check if they are equal.
    line_content == added_line_content
}

fn find_replacements(deleted_content: Vec<String>, added_content: Vec<String>) -> Vec<u32> {
    // Find the replacements in the deleted content and added content.
    // For now, just return the added content.
    let mut replaced_content_line_numbers = vec![];
    let deleted_content_iter = deleted_content.iter().enumerate();
    for (_, line_content) in deleted_content_iter {
        for (idx_add, line_add_content) in added_content.iter().enumerate() {
            if is_similar(line_content, line_add_content) {
                replaced_content_line_numbers.push(idx_add as u32);
            }
        }
    }
    replaced_content_line_numbers
}

#[derive(PartialEq, Eq, Debug)]
enum DiffCases {
    FewLinesReplacedWithSingleLine, // -2,5 +2
    FewLinesReplacedWithFewLines,
    SingleLineDeleted,
    FewLinesDeleted,
    SingleLineReplacedWithAnotherSingleLine,
    NewLinesAdded,
    NoneFound,
}

fn read_content(
    all_lines: &mut str::Lines,
    deleted_line_count: u32,
    added_line_count: u32,
    map_to_fill: &mut HashMap<u32, Vec<LineDetail>>,
    start_line_number_if_to_add: Option<u32>,
) -> (Vec<String>, Vec<String>) {
    let mut deleted_content = vec![];
    // While iterating - also make sure that you're filling up map_to_fill: with a LineDetail
    // entry.
    for _ in 0..deleted_line_count {
        if let Some(line) = all_lines.next() {
            deleted_content.push(line.to_string());
        }
    }
    let mut added_content = vec![];
    for _ in 0..added_line_count {
        if let Some(line) = all_lines.next() {
            added_content.push(line.to_string());
            if let Some(start_line_number) = start_line_number_if_to_add {
                // If we have a start line number, then we need to add the content to the map.
                // This is only for cases when NEW lines are added.
                map_to_fill.insert(
                    start_line_number,
                    vec![LineDetail {
                        content: line.to_string(),
                        commit_hashes: vec![],
                    }],
                );
            }
        }
    }
    (deleted_content, added_content)
}

fn categorize_diff(line: &str) -> Option<DiffCases> {
    let re = regex::Regex::new(r"^-([0-9]+)(?:,([0-9]+))?\s+\+([0-9]+)(?:,([0-9]+))?$")
        .expect("Invalid regex");

    let caps = re.captures(line)?;

    let minus_count = caps
        .get(2)
        .map_or(1, |m| m.as_str().parse::<usize>().unwrap_or(1));
    let plus_count = caps
        .get(4)
        .map_or(1, |m| m.as_str().parse::<usize>().unwrap_or(1));

    match (minus_count, plus_count) {
        (m, 1) if m > 1 => Some(DiffCases::FewLinesReplacedWithSingleLine),
        (m, p) if m > 1 && p > 1 => Some(DiffCases::FewLinesReplacedWithFewLines),
        (1, 0) => Some(DiffCases::SingleLineDeleted),
        (m, 0) if m > 1 => Some(DiffCases::FewLinesDeleted),
        (1, 1) => Some(DiffCases::SingleLineReplacedWithAnotherSingleLine),
        (0, p) if p > 0 => Some(DiffCases::NewLinesAdded),
        _ => Some(DiffCases::NoneFound),
    }
}

fn reorder_map(
    commit_hash: String,
    category: Option<DiffCases>,
    map: &mut HashMap<u32, Vec<LineDetail>>,
    line_change_before: LineChange,
    line_change_after: LineChange,
    replaced_content_line_numbers: Vec<u32>,
) {
    match category {
        Some(DiffCases::FewLinesReplacedWithSingleLine) => {
            // That means, anything after the current index, should be subtracted accordingly.
            // Modify the line numbers in the map.
            // Check if the lines replaced - if any of those were closely related to the single
            // line, if yes, then we store the commit hash into that hashmap for that line number,
            // otherwise we delete the content from the hashmap and stop tracking that.

            // For now, just assume that they aren't similar at all. And just delete the entries
            // from the HashMap and revise entries post it.
            let s_line_no = line_change_before.start_line_number;
            let e_line_no = line_change_before.start_line_number + line_change_before.change_count;
            let map_len = map.len();

            // Inclusive e_line_no - 1;
            for l_no in (s_line_no)..=(e_line_no - 1) {
                // Replaced line content numbers means that this line was "replaced" and not
                // removed. So, in this case - do not remove content from the map.
                // Later on, we'll append the commit hash.
                if !replaced_content_line_numbers.contains(&l_no) {
                    map.remove(&l_no);
                } else {
                    map.get_mut(&l_no).map(|line_details| {
                        line_details[0].commit_hashes.push(commit_hash.clone());
                        // The content to replace with would be (l_no - s_line_no)th index in
                        // line_change_after.changed_content.
                        line_details[0].content = line_change_after
                            .changed_content
                            .get((l_no - s_line_no) as usize)
                            .unwrap()
                            .to_string();
                    });
                }
            }

            // Now update all the lines in the hash map and shift them:
            let mut to_remove_map: HashMap<u32, Vec<LineDetail>> = HashMap::new();
            for l_no in map.keys().cloned().collect::<Vec<u32>>() {
                if l_no >= e_line_no {
                    let new_idx = l_no - (line_change_before.change_count - 1);
                    if new_idx >= s_line_no && new_idx < e_line_no {
                        let line_detail_to_replace_with = map.get(&l_no).unwrap()[0].clone();
                        to_remove_map.insert(
                            new_idx,
                            vec![LineDetail {
                                content: line_detail_to_replace_with.content,
                                commit_hashes: vec![commit_hash.clone()],
                            }],
                        );
                        continue;
                    }
                    let to_remove = map.remove(&new_idx);
                    if to_remove.is_none() {
                        // Post this, there's nothing to find.
                        panic!("Line number {} not found in map", new_idx);
                    }
                    to_remove_map.insert(new_idx, to_remove.unwrap());
                }
            }

            // Now insert the new entries.
            for (l_no, line_detail) in to_remove_map.iter() {
                println!(
                    "Inserting: {} content: {}",
                    l_no,
                    line_detail[0].content.clone()
                );
                map.insert(*l_no, line_detail.to_vec());
            }

            // For the first line that just got replaced, create a new entry.
            // map.insert(
            //     s_line_no,
            //     vec![LineDetail {
            //         content: "New Content".to_string(), // FIXME: We don't have content yet. This is bad?
            //         commit_hashes: vec![commit_hash],
            //     }],
            // );
            // In the final map.keys(), delete last line_change_before.change_count - 1 entries - because they
            // are already shifted by that count.
            for key in map.keys().cloned().collect::<Vec<u32>>() {
                if key > (map_len as u32 - (line_change_before.change_count - 1)) {
                    map.remove(&key);
                }
            }
        }
        Some(DiffCases::FewLinesReplacedWithFewLines) => {
            // Always use line_change_after to begin with as that is the source of truth.
            let s_line_no = line_change_before.start_line_number;
            let e_line_no = line_change_after.start_line_number + line_change_before.change_count;

            let diff =
                line_change_after.change_count as i32 - line_change_before.change_count as i32;
            if diff > 0 {
                // Lines deleted < Lines added.
                // First move all the lines after e_line_no+diff.
                let mut to_remove_map: HashMap<u32, Vec<LineDetail>> = HashMap::new();
                for l_no in map.keys().cloned().collect::<Vec<u32>>() {
                    if l_no >= e_line_no {
                        let new_idx = l_no + (diff as u32);
                        let to_remove = map.remove(&l_no);
                        if to_remove.is_none() {
                            // Post this, there's nothing to find.
                            panic!("Line number {} not found in map", l_no);
                        }
                        to_remove_map.insert(new_idx, to_remove.unwrap());
                    }
                }

                // Now insert the new entries.
                for (l_no, line_detail) in to_remove_map {
                    map.insert(l_no, line_detail);
                }

                // We need to add lines.
                let e_line_no =
                    line_change_after.start_line_number + line_change_after.change_count;
                for l_no in s_line_no..e_line_no {
                    map.remove(&l_no);
                    // Insert new entries again for these lines.
                    map.insert(
                        l_no,
                        vec![LineDetail {
                            content: "New Content".to_string(), // FIXME: We don't have content yet. This is bad?
                            commit_hashes: vec![commit_hash.clone()],
                        }],
                    );
                }
            } else {
                // Lines deleted > Lines added.
                for l_no in s_line_no..=(e_line_no + 1) {
                    map.remove(&l_no);
                }

                // Add content for the new lines.
                for l_no in s_line_no
                    ..=(line_change_after.start_line_number + line_change_after.change_count + 1)
                {
                    map.insert(
                        l_no,
                        vec![LineDetail {
                            content: "New Content".to_string(), // FIXME: We don't have content yet. This is bad?
                            commit_hashes: vec![commit_hash.clone()],
                        }],
                    );
                }

                // Now for all the lines in the map that are > line_change_after.start_line_number
                // + line_change_after.change_count, move them by -diff.
                for l_no in map.keys().cloned().collect::<Vec<u32>>() {
                    if l_no > line_change_after.start_line_number + line_change_after.change_count {
                        let new_idx: i32 = l_no as i32 + diff; // diff is negative here.
                        let to_remove = map.remove(&(new_idx as u32));
                        if to_remove.is_none() {
                            panic!("Line number {} not found in map", new_idx);
                        }
                        map.insert(new_idx as u32, to_remove.unwrap());
                    }
                }
            }
        }
        Some(DiffCases::SingleLineDeleted) => {
            // This is simple, just delete the recording of the given line, and shift the rest of
            // the code by -1.
            let s_line_no = line_change_after.start_line_number;
            map.remove(&s_line_no);
            // Now move everything that is >= s_line_no, shift left.
            // Iterate in the sorted order.
            let mut to_remove_map: HashMap<u32, Vec<LineDetail>> = HashMap::new();
            for l_no in map.keys().cloned().collect::<Vec<u32>>() {
                if l_no > s_line_no {
                    let new_idx = l_no - 1;
                    let to_remove = map.remove(&l_no);
                    if to_remove.is_none() {
                        // Post this, there's nothing to find.
                        panic!("Line number {} not found in map", l_no);
                    }
                    to_remove_map.insert(new_idx, to_remove.unwrap());
                }
            }
            // Now insert the new entries.
            for (l_no, line_detail) in to_remove_map {
                map.insert(l_no, line_detail);
            }
            // Now just remove the last line.
            let map_len = map.len();
            map.remove(&(map_len as u32));
        }
        Some(DiffCases::FewLinesDeleted) => {
            let s_line_no = line_change_after.start_line_number;
            let e_line_no = line_change_after.start_line_number + line_change_before.change_count;

            // Remove all lines b/w s_line_no and e_line_no (exclusive).
            for l_no in s_line_no..e_line_no {
                map.remove(&l_no);
            }
            // For any line after e_line_no, shift them by line_change_before.change_count.
            let mut to_remove_map: HashMap<u32, Vec<LineDetail>> = HashMap::new();
            for l_no in map.keys().cloned().collect::<Vec<u32>>() {
                if l_no >= e_line_no {
                    let new_idx = l_no - line_change_before.change_count;
                    let to_remove = map.remove(&l_no);
                    if to_remove.is_none() {
                        // Post this, there's nothing to find.
                        panic!("Line number {} not found in map", l_no);
                    }
                    to_remove_map.insert(new_idx, to_remove.unwrap());
                }
            }
            // Now insert the new entries.
            for (l_no, line_detail) in to_remove_map {
                map.insert(l_no, line_detail);
            }
        }
        Some(DiffCases::SingleLineReplacedWithAnotherSingleLine) => {
            let s_line_no = line_change_after.start_line_number;
            // Replace the line with the new line.
            let to_remove = map.get_mut(&s_line_no);
            if to_remove.is_none() {
                // Post this, there's nothing to find.
                panic!("Line number {} not found in map", s_line_no);
            }
            let line_detail = to_remove.unwrap();
            line_detail[0].content = "New Content".to_string(); // FIXME: We don't have content
            line_detail[0].commit_hashes = vec![commit_hash];
        }
        Some(DiffCases::NewLinesAdded) => {
            // Handle this case
            let s_line_no = line_change_after.start_line_number;
            let e_line_no = line_change_after.start_line_number + line_change_after.change_count;

            // Anything after e_line_no, should be moved by line_change_before.change_count;
            let mut to_remove_map: HashMap<u32, Vec<LineDetail>> = HashMap::new();
            for l_no in map.keys().cloned().collect::<Vec<u32>>() {
                if l_no >= e_line_no {
                    let new_idx = l_no + line_change_before.change_count;
                    let to_remove = map.remove(&l_no);
                    if to_remove.is_none() {
                        // Post this, there's nothing to find.
                        panic!("Line number {} not found in map", l_no);
                    }
                    to_remove_map.insert(new_idx, to_remove.unwrap());
                }
            }

            // Now insert the new entries.
            for (l_no, line_detail) in to_remove_map {
                map.insert(l_no, line_detail);
            }
            // Now add the new lines.
            for l_no in s_line_no..=e_line_no {
                map.remove(&l_no);
                map.insert(
                    l_no,
                    vec![LineDetail {
                        content: "New Content".to_string(), // FIXME: We don't have content yet. This is bad?
                        commit_hashes: vec![commit_hash.clone()],
                    }],
                );
            }
        }
        Some(DiffCases::NoneFound) => {
            // Handle this case
        }
        _ => {}
    }
}

fn fetch_line_numbers(line: String) -> (LineChange, LineChange) {
    // Extract the line numbers from the diff line.
    // The line format is expected to be like:
    // -2,5 +2
    // -90,61 +86
    // -2,5 +2,3
    // -23,2 +18,7
    // -45 +44,0
    // -50,3 +48,0
    // -159 +96
    let re = regex::Regex::new(r"^-(\d+)(?:,(\d+))?\s+\+(\d+)(?:,(\d+))?$").unwrap();
    if let Some(caps) = re.captures(&line) {
        let line_before = caps[1].parse::<u32>().unwrap();
        let line_after = caps[3].parse::<u32>().unwrap();

        let line_before_count = if caps.get(2).is_some() {
            caps[2].parse::<u32>().unwrap()
        } else {
            1
        };

        let line_after_count = if caps.get(4).is_some() {
            caps[4].parse::<u32>().unwrap()
        } else {
            1
        };

        (
            LineChange {
                start_line_number: line_before,
                change_type: ChangeType::Deleted,
                change_count: line_before_count,
                changed_content: vec![],
            },
            LineChange {
                start_line_number: line_after,
                change_type: ChangeType::Added,
                change_count: line_after_count,
                changed_content: vec![],
            },
        )
    } else {
        panic!("Invalid line format: {}", line);
    }
}

fn parse_diff(
    commit_hash: String,
    commit_diff: String,
    map: &mut HashMap<u32, Vec<LineDetail>>,
) -> Result<(), String> {
    // Cases possible:
    //
    // -2,5 +2 -> few lines were replaced with a single line.
    // -90,61 +86 -> few lines were replaced with a single line.
    // -2,5 +2,3 -> few lines were replaced with a few lines.
    // -23,2 +18,7 -> (same as above) few lines were replaced with a few lines.
    // -45 +44,0 -> a single line was deleted.
    // -50,3 +48,0 -> a few lines were deleted.
    // -159 +96 -> a single line was replaced with another single line.
    // -169,0 +104,3 -> new lines were added.
    let mut all_lines = commit_diff.lines();
    while true {
        let line = all_lines.next();
        if line.is_none() {
            break;
        }
        let line = line.unwrap();
        let line = line.trim();
        let mut line_before: Option<LineChange> = None;
        let mut line_after: Option<LineChange> = None;
        let mut category: Option<DiffCases> = None;
        if line.starts_with("@@") {
            // Only process till the last '@@'
            // TODO: Do this using a regex instead.
            let mut line = line.to_string();
            line = line.split_once("@@").unwrap().1.to_string();
            line = line.split_once("@@").unwrap().0.trim().to_string();
            let line_changes = fetch_line_numbers(line.clone());
            line_before = Some(line_changes.0);
            line_after = Some(line_changes.1);
            category = categorize_diff(line.as_str());
            // After this, we have to go on until line_changes.change_count -> those are deleted
            // lines if change_type == ChangeType::Deleted, otherwise they are the newly added
            // lines if change_type == ChangeType::Addded.
            // reorder_map(commit_hash.clone(), category, map, line_before, line_after);
        }
        if line_before.is_some() && line_after.is_some() {
            if map.is_empty() {
                let l_after = line_after.unwrap();
                _ = read_content(
                    &mut all_lines,
                    line_before.unwrap().change_count,
                    l_after.change_count,
                    map,
                    Some(l_after.start_line_number),
                );
            } else {
                let content = read_content(
                    &mut all_lines,
                    line_before.clone().unwrap().change_count,
                    line_after.clone().unwrap().change_count,
                    map,
                    None,
                );
                let deleted_content = content.0;
                let added_content = content.1;
                // In any case -> line_after should have the content of the new lines.
                // So make sure to replace here with added_content:
                let line_after = line_after.clone();
                line_after.clone().unwrap().changed_content = added_content.clone();
                let mut replaced_content_line_numbers =
                    find_replacements(deleted_content, added_content);
                // Now for each of these replaced_content_line_numbers - add start_line_number from
                // line_after, as they were just indices before.
                let l_before_start_line_no = line_before.clone().unwrap().start_line_number;
                replaced_content_line_numbers.iter_mut().for_each(|x| {
                    *x = l_before_start_line_no + *x;
                });
                reorder_map(
                    commit_hash.clone(),
                    category,
                    map,
                    line_before.unwrap(),
                    line_after.unwrap(),
                    replaced_content_line_numbers,
                );
            }
        }
        // After this, next line_before.change_count lines are the ones that are related to the
        // change before.
        // And after it, next line_after.change_count lines are the ones that are related to the
        // change after.
        // Parse the next line_before.change_count lines.
        //     let mut before_content: Vec<String> = vec![];
        //     let mut after_content: Vec<String> = vec![];
        //     while let Some(line) = commit_diff.lines().next() {
        //         let line = line.trim();
        //         if line.starts_with("@@") {
        //             break;
        //         }
        //         if line.starts_with('-') {
        //             // Deleted lines.
        //             let line_no = line[1..].parse::<u32>().unwrap();
        //             let content = line[1..].to_string();
        //             map.insert(
        //                 line_no,
        //                 vec![LineDetail {
        //                     content: content.clone(),
        //                     commit_hashes: vec![commit_hash.clone()],
        //                 }],
        //             );
        //             before_content.push(content);
        //         } else if line.starts_with('+') {
        //             // Added lines.
        //             let line_no = line[1..].parse::<u32>().unwrap();
        //             let content = line[1..].to_string();
        //             map.insert(
        //                 line_no,
        //                 vec![LineDetail {
        //                     content: content.clone(),
        //                     commit_hashes: vec![commit_hash.clone()],
        //                 }],
        //             );
        //             after_content.push(content);
        //         }
        //     }
        //     if line_before.is_none() || line_after.is_none() {
        //         // None found, just keep going.
        //         continue;
        //     }
        //     // line_before.unwrap().changed_content = before_content;
        //     // line_after.unwrap().changed_content = after_content;
        //     // Change line_before's changed_content to before_content:
        //
        //     let modified_line_before = LineChange {
        //         line_before.unwrap().start_line_number,
        //     }
        //     reorder_map(
        //         commit_hash.clone(),
        //         category,
        //         map,
        //         l_before.unwrap(),
        //         line_after.unwrap()
        //     );
    }
    Ok(())
}

// fn extract_commit_hashes(commit_hash) {
//     // Call git show --unified=0 for the commit_hash and extract line->[commit_hash...] list.
//     let output = std::process::Command::new("git")
//         .arg("show")
//         .arg("--unified=0")
//         .arg(commit_hash)
//         .output()
//         .expect("Failed to execute command");
//     if output.status.success() {
//         let stdout = String::from_utf8_lossy(&output.stdout);
//     } else {
//         eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
//     }
// }

#[cfg(test)]
mod tests_diff_v2 {
    use super::*;

    #[test]
    fn test_few_lines_replaced_with_single_line() {
        assert_eq!(
            categorize_diff("-2,5 +2"),
            Some(DiffCases::FewLinesReplacedWithSingleLine)
        );
    }

    #[test]
    fn test_few_lines_replaced_with_few_lines() {
        assert_eq!(
            categorize_diff("-2,5 +2,3"),
            Some(DiffCases::FewLinesReplacedWithFewLines)
        );
    }

    #[test]
    fn test_single_line_deleted() {
        assert_eq!(
            categorize_diff("-45 +44,0"),
            Some(DiffCases::SingleLineDeleted)
        );
    }

    #[test]
    fn test_few_lines_deleted() {
        assert_eq!(
            categorize_diff("-50,3 +48,0"),
            Some(DiffCases::FewLinesDeleted)
        );
    }

    #[test]
    fn test_single_line_replaced_with_another_single_line() {
        assert_eq!(
            categorize_diff("-159 +96"),
            Some(DiffCases::SingleLineReplacedWithAnotherSingleLine)
        );
    }

    #[test]
    fn test_new_lines_added() {
        assert_eq!(
            categorize_diff("-169,0 +104,3"),
            Some(DiffCases::NewLinesAdded)
        );
    }

    #[test]
    fn test_invalid_format() {
        assert_eq!(categorize_diff("bad input"), None);
    }

    // #[test]
    // fn test_full_commit_diff() {
    //     let commit_diff = r#"
    //         @@ -2,5 +2 @@
    //         -line1
    //         -line2
    //         +line3
    //         @@ -90,61 +86 @@
    //         -line4
    //         +line5
    //         @@ -2,5 +2,3 @@
    //         -line6
    //         +line7
    //         @@ -23,2 +18,7 @@
    //         -line8
    //         +line9
    //         @@ -45 +44,0 @@
    //         -line10
    //         @@ -50,3 +48,0 @@
    //         -line11
    //         @@ -159 +96 @@
    //         -line12
    //     "#;
    //
    //     let commit_hash = "";
    //
    //     let mut map: HashMap<u32, Vec<LineDetail>> = HashMap::new();
    //     parse_diff(commit_hash.to_string(), commit_diff.to_string(), &mut map).unwrap();
    // }

    #[test]
    fn test_fetch_line_numbers_replace_few_lines_with_single_line() {
        let line = "-2,5 +2";
        let (line_before, line_after) = fetch_line_numbers(line.to_string());
        assert_eq!(
            line_before,
            LineChange {
                start_line_number: 2,
                change_count: 5,
                change_type: ChangeType::Deleted,
                changed_content: vec![]
            }
        );
        assert_eq!(
            line_after,
            LineChange {
                start_line_number: 2,
                change_count: 1,
                change_type: ChangeType::Added,
                changed_content: vec![]
            }
        );
    }

    #[test]
    fn test_fetch_line_numbers_replace_few_lines_with_few_lines() {
        let line = "-2,5 +2,3";
        let (line_before, line_after) = fetch_line_numbers(line.to_string());
        assert_eq!(
            line_before,
            LineChange {
                start_line_number: 2,
                change_count: 5,
                change_type: ChangeType::Deleted,
                changed_content: vec![]
            }
        );
        assert_eq!(
            line_after,
            LineChange {
                start_line_number: 2,
                change_count: 3,
                change_type: ChangeType::Added,
                changed_content: vec![]
            }
        );
    }

    #[test]
    fn test_fetch_line_numbers_single_line_deleted() {
        let line = "-45 +44,0";
        let (line_before, line_after) = fetch_line_numbers(line.to_string());
        assert_eq!(
            line_before,
            LineChange {
                start_line_number: 45,
                change_count: 1,
                change_type: ChangeType::Deleted,
                changed_content: vec![]
            }
        );
        assert_eq!(
            line_after,
            LineChange {
                start_line_number: 44,
                change_count: 0,
                change_type: ChangeType::Added,
                changed_content: vec![]
            }
        );
    }

    #[test]
    fn test_fetch_line_numbers_few_lines_deleted() {
        let line = "-50,3 +48,0";
        let (line_before, line_after) = fetch_line_numbers(line.to_string());
        assert_eq!(
            line_before,
            LineChange {
                start_line_number: 50,
                change_count: 3,
                change_type: ChangeType::Deleted,
                changed_content: vec![]
            }
        );
        assert_eq!(
            line_after,
            LineChange {
                start_line_number: 48,
                change_count: 0,
                change_type: ChangeType::Added,
                changed_content: vec![]
            }
        );
    }

    #[test]
    fn test_fetch_line_numbers_single_line_replaced_with_another_single_line() {
        let line = "-159 +96";
        let (line_before, line_after) = fetch_line_numbers(line.to_string());
        assert_eq!(
            line_before,
            LineChange {
                start_line_number: 159,
                change_count: 1,
                change_type: ChangeType::Deleted,
                changed_content: vec![]
            }
        );
        assert_eq!(
            line_after,
            LineChange {
                start_line_number: 96,
                change_count: 1,
                change_type: ChangeType::Added,
                changed_content: vec![]
            }
        );
    }

    #[test]
    fn test_fetch_line_numbers_new_lines_added() {
        let line = "-169,0 +104,3";
        let (line_before, line_after) = fetch_line_numbers(line.to_string());
        assert_eq!(
            line_before,
            LineChange {
                start_line_number: 169,
                change_count: 0,
                change_type: ChangeType::Deleted,
                changed_content: vec![]
            }
        );
        assert_eq!(
            line_after,
            LineChange {
                start_line_number: 104,
                change_count: 3,
                change_type: ChangeType::Added,
                changed_content: vec![]
            }
        );
    }

    #[test]
    #[should_panic(expected = "Invalid line format: bad input")]
    fn test_fetch_line_numbers_invalid_format() {
        let line = "bad input"; // Invalid format.
        fetch_line_numbers(line.to_string());
    }

    #[test]
    fn test_reorder_map_few_lines_replaced_with_single_line() {
        // Test that the map is correctly reordered for the case below:
        // -2,5 +2
        // Make sure that 6th line in the map is now 8th line, because we have "added" 7 lines
        // above.
        let mut map: HashMap<u32, Vec<LineDetail>> = HashMap::new();
        map.insert(
            1,
            vec![LineDetail {
                content: "line1".to_string(),
                commit_hashes: vec!["commit1".to_string()],
            }],
        );
        map.insert(
            2,
            vec![LineDetail {
                content: "line2".to_string(),
                commit_hashes: vec!["commit1".to_string()],
            }],
        );
        map.insert(
            3,
            vec![LineDetail {
                content: "line3".to_string(),
                commit_hashes: vec!["commit1".to_string()],
            }],
        );
        map.insert(
            4,
            vec![LineDetail {
                content: "line4".to_string(),
                commit_hashes: vec!["commit1".to_string()],
            }],
        );
        map.insert(
            5,
            vec![LineDetail {
                content: "line5".to_string(),
                commit_hashes: vec!["commit1".to_string()],
            }],
        );
        map.insert(
            6,
            vec![LineDetail {
                content: "line6".to_string(),
                commit_hashes: vec!["commit1".to_string()],
            }],
        );
        map.insert(
            7,
            vec![LineDetail {
                content: "line7".to_string(),
                commit_hashes: vec!["commit1".to_string()],
            }],
        );
        map.insert(
            8,
            vec![LineDetail {
                content: "line8".to_string(),
                commit_hashes: vec!["commit1".to_string()],
            }],
        );
        map.insert(
            9,
            vec![LineDetail {
                content: "line9".to_string(),
                commit_hashes: vec!["commit1".to_string()],
            }],
        );
        reorder_map(
            "commit2".to_string(),
            Some(DiffCases::FewLinesReplacedWithSingleLine),
            &mut map,
            LineChange {
                start_line_number: 2,
                change_count: 5,
                change_type: ChangeType::Deleted,
                changed_content: vec![
                    "line2".to_string(),
                    "line3".to_string(),
                    "line4".to_string(),
                    "line5".to_string(),
                    "line5".to_string(),
                ],
            },
            LineChange {
                start_line_number: 2,
                change_count: 1,
                change_type: ChangeType::Added,
                changed_content: vec!["line23".to_string()],
            },
            vec![2],
        );

        // Make sure that map keys are correct.
        assert_eq!(map.len(), 5);
        // Make sure that for line number (2), the content is unchanged.
        assert_eq!(map.get(&2).unwrap()[0].content, "line23".to_string());

        // For 2nd line number, we should have commit1 and commit2.
        assert_eq!(
            map.get(&2).unwrap()[0].commit_hashes,
            vec!["commit1".to_string(), "commit2".to_string()]
        );
        // Make sure that from 2 to 8 (inclusive), the content's commit hash is now commit2:
        for i in 3..=4 {
            assert_eq!(
                map.get(&i).unwrap()[0].commit_hashes[0],
                "commit2".to_string()
            );
        }
        assert_eq!(map.get(&2).unwrap()[0].content, "line23".to_string()); // New content.
        assert_eq!(map.get(&3).unwrap()[0].content, "line7".to_string());
        assert_eq!(map.get(&4).unwrap()[0].content, "line8".to_string());
        assert_eq!(map.get(&5).unwrap()[0].content, "line9".to_string());
    }

    // #[test]
    // fn test_reorder_map() {
    //     // Test that the map is correctly reordered for the case below:
    //     // -2,5 +2,7
    //     // Make sure that 6th line in the map is now 8th line, because we have "added" 7 lines
    //     // above.
    //     let mut map: HashMap<u32, Vec<LineDetail>> = HashMap::new();
    //     map.insert(
    //         1,
    //         vec![LineDetail {
    //             content: "line1".to_string(),
    //             commit_hashes: vec!["commit1".to_string()],
    //         }],
    //     );
    //     map.insert(
    //         2,
    //         vec![LineDetail {
    //             content: "line2".to_string(),
    //             commit_hashes: vec!["commit1".to_string()],
    //         }],
    //     );
    //     map.insert(
    //         3,
    //         vec![LineDetail {
    //             content: "line3".to_string(),
    //             commit_hashes: vec!["commit1".to_string()],
    //         }],
    //     );
    //     map.insert(
    //         4,
    //         vec![LineDetail {
    //             content: "line4".to_string(),
    //             commit_hashes: vec!["commit1".to_string()],
    //         }],
    //     );
    //     map.insert(
    //         5,
    //         vec![LineDetail {
    //             content: "line5".to_string(),
    //             commit_hashes: vec!["commit1".to_string()],
    //         }],
    //     );
    //     map.insert(
    //         6,
    //         vec![LineDetail {
    //             content: "line6".to_string(),
    //             commit_hashes: vec!["commit1".to_string()],
    //         }],
    //     );
    //     map.insert(
    //         7,
    //         vec![LineDetail {
    //             content: "line7".to_string(),
    //             commit_hashes: vec!["commit1".to_string()],
    //         }],
    //     );
    //     map.insert(
    //         8,
    //         vec![LineDetail {
    //             content: "line8".to_string(),
    //             commit_hashes: vec!["commit1".to_string()],
    //         }],
    //     );
    //     map.insert(
    //         9,
    //         vec![LineDetail {
    //             content: "line9".to_string(),
    //             commit_hashes: vec!["commit1".to_string()],
    //         }],
    //     );
    //     reorder_map(
    //         "commit2".to_string(),
    //         Some(DiffCases::FewLinesReplacedWithFewLines),
    //         &mut map,
    //         LineChange {
    //             start_line_number: 2,
    //             change_count: 5,
    //             change_type: ChangeType::Deleted,
    //         },
    //         LineChange {
    //             start_line_number: 2,
    //             change_count: 7,
    //             change_type: ChangeType::Added,
    //         },
    //     );

    //     // Make sure that map keys are correct.
    //     assert_eq!(map.len(), 11);
    //     // Make sure that for 1st line number, the content is unchanged.
    //     assert_eq!(map.get(&1).unwrap()[0].content, "line1".to_string());

    //     // Make sure that from 2 to 8 (inclusive), the content's commit hash is now commit2:
    //     for i in 2..=8 {
    //         assert_eq!(
    //             map.get(&i).unwrap()[0].commit_hashes[0],
    //             "commit2".to_string()
    //         );
    //     }

    //     // From 9 to 11 (inclusive), the content is the same as previous map, that is commit hashes
    //     // are commit1.
    //     assert_eq!(map.get(&9).unwrap()[0].content, "line7".to_string());
    //     assert_eq!(map.get(&10).unwrap()[0].content, "line8".to_string());
    //     assert_eq!(map.get(&11).unwrap()[0].content, "line9".to_string());
    // }

    // #[test]
    // fn test_multiple_commits_reorder_map() {
    //     // Test for commit that goes like:
    //     // -2,5 +2
    //     // -8 +3,0
    //     // -23,2 +18,7
    //     let mut map: HashMap<u32, Vec<LineDetail>> = HashMap::new();
    //     // Create map at ocne.
    //     for i in 1..=25 {
    //         map.insert(
    //             i,
    //             vec![LineDetail {
    //                 content: format!("line{}", i),
    //                 commit_hashes: vec!["commit1".to_string()],
    //             }],
    //         );
    //     }
    //     assert_eq!(map.len(), 25);
    //     // Now, we need to reorder the map for the first commit.
    //     reorder_map(
    //         "commit2".to_string(),
    //         Some(DiffCases::FewLinesReplacedWithSingleLine),
    //         &mut map,
    //         LineChange {
    //             start_line_number: 2,
    //             change_count: 5,
    //             change_type: ChangeType::Deleted,
    //         },
    //         LineChange {
    //             start_line_number: 2,
    //             change_count: 1,
    //             change_type: ChangeType::Added,
    //         },
    //     );
    //     // First make sure that this worked as expected.
    //     assert_eq!(map.len(), 21);

    //     // Now try -8 +3,0
    //     reorder_map(
    //         "commit3".to_string(),
    //         Some(DiffCases::SingleLineDeleted),
    //         &mut map,
    //         LineChange {
    //             start_line_number: 8,
    //             change_count: 1,
    //             change_type: ChangeType::Deleted,
    //         },
    //         LineChange {
    //             start_line_number: 3,
    //             change_count: 0,
    //             change_type: ChangeType::Added,
    //         },
    //     );
    //     // Added one line, deleted one line.
    //     assert_eq!(map.len(), 19);

    //     // Now try for -23,2 +18,7
    //     reorder_map(
    //         "commit4".to_string(),
    //         Some(DiffCases::FewLinesReplacedWithFewLines),
    //         &mut map,
    //         LineChange {
    //             start_line_number: 23,
    //             change_count: 2,
    //             change_type: ChangeType::Deleted,
    //         },
    //         LineChange {
    //             start_line_number: 18,
    //             change_count: 7,
    //             change_type: ChangeType::Added,
    //         },
    //     );
    //     assert_eq!(map.len(), 24);
    //     assert_eq!(map.get(&19).unwrap()[0].content, "New Content".to_string());
    //     assert_eq!(map.get(&20).unwrap()[0].content, "New Content".to_string());

    //     // Now check for the case with "few lines deleted".
    //     // Trying for -26,3 +25,0
    //     reorder_map(
    //         "commit4".to_string(),
    //         Some(DiffCases::FewLinesDeleted),
    //         &mut map,
    //         LineChange {
    //             start_line_number: 22,
    //             change_count: 3,
    //             change_type: ChangeType::Deleted,
    //         },
    //         LineChange {
    //             start_line_number: 21,
    //             change_count: 0,
    //             change_type: ChangeType::Added,
    //         },
    //     );
    //     assert_eq!(map.len(), 21);
    //     // Make sure that data for any line after the deleted lines is retained.
    // assert_eq!(map.get(&19).unwrap()[0].content, "New Content".to_string());
    // }
}
