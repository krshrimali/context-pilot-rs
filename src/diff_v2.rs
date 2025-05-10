use std::{collections::HashMap, str};

#[derive(Debug, Clone)]
pub struct LineDetail {
    pub content: String,
    pub commit_hashes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeType {
    Added,
    Deleted,
    Modified,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineChange {
    pub start_line_number: u32,
    pub change_count: u32,
    pub change_type: ChangeType,
    pub changed_content: Vec<String>,
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
pub enum DiffCases {
    FewLinesReplacedWithSingleLine, // -2,5 +2
    FewLinesReplacedWithFewLines,
    SingleLineDeleted,
    FewLinesDeleted,
    SingleLineReplacedWithAnotherSingleLine,
    NewLinesAdded,
    NoneFound,
}

pub fn read_content(
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

pub fn categorize_diff(line: &str) -> Option<DiffCases> {
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

pub fn reorder_map(
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
            let s_line_no = line_change_after.start_line_number;
            let e_line_no = line_change_after.start_line_number + line_change_before.change_count;
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
                                commit_hashes: line_detail_to_replace_with.commit_hashes,
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
                    "Inserting: {} content: {}, with commit hash: {:?}",
                    l_no,
                    line_detail[0].content.clone(),
                    line_detail[0].commit_hashes.clone()
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
            let s_line_no = line_change_after.start_line_number;
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
                        println!(
                            "For l_no: {} to new_idx: {}, content: {:?},  {:?}",
                            l_no,
                            new_idx,
                            to_remove.clone().unwrap()[0].content,
                            to_remove.clone().unwrap()[0].commit_hashes
                        );
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
                    let new_content = line_change_after
                        .changed_content
                        .get((l_no - s_line_no) as usize)
                        .unwrap()
                        .to_string();
                    if replaced_content_line_numbers.contains(&l_no) {
                        println!("line was replaced");
                        // This line was replaced and not deleted -> and then added.
                        map.get_mut(&l_no).map(|line_details| {
                            line_details[0].commit_hashes.push(commit_hash.clone());
                            // Line content to replace with would be (l_no-s_line_no)th index
                            // in line_change_after.changed_content.
                            line_details[0].content = new_content.clone();
                        });
                    } else {
                        // Added new line:
                        map.remove(&l_no);
                        // Insert new entries again for these lines.
                        map.insert(
                            l_no,
                            vec![LineDetail {
                                content: new_content,
                                commit_hashes: vec![commit_hash.clone()],
                            }],
                        );
                    }
                }
            } else {
                // Lines deleted > Lines added.
                for l_no in s_line_no..=e_line_no {
                    if replaced_content_line_numbers.contains(&l_no) {
                        println!("For l_no: {}", l_no);
                        println!("{:?}", map.get(&l_no).unwrap()[0].commit_hashes);
                        let new_content = line_change_after
                            .changed_content
                            .get((l_no - s_line_no) as usize)
                            .unwrap()
                            .to_string();
                        map.get_mut(&l_no).map(|line_details| {
                            line_details[0].commit_hashes.push(commit_hash.clone());
                            line_details[0].content = new_content.clone();
                        });
                        println!("{:?}", map.get(&l_no).unwrap()[0].commit_hashes);
                    }
                }

                for l_no in s_line_no..e_line_no {
                    if !replaced_content_line_numbers.contains(&l_no) {
                        println!("Removing line: {:?}", l_no);
                        map.remove(&l_no);
                    }
                }

                // Add content for the new lines.
                for l_no in s_line_no
                    ..(line_change_after.start_line_number + line_change_after.change_count)
                {
                    if !replaced_content_line_numbers.contains(&l_no) {
                        let new_content = line_change_after
                            .changed_content
                            .get((l_no - s_line_no) as usize)
                            .unwrap()
                            .to_string();
                        map.insert(
                            l_no,
                            vec![LineDetail {
                                content: new_content.clone(),
                                commit_hashes: vec![commit_hash.clone()],
                            }],
                        );
                    }
                }

                // Now for all the lines in the map that are > line_change_after.start_line_number
                // + line_change_after.change_count, move them by -diff.
                for l_no in map.keys().cloned().collect::<Vec<u32>>() {
                    if l_no >= line_change_after.start_line_number + line_change_after.change_count
                    {
                        let new_idx: i32 = l_no as i32 + diff; // diff is negative here.
                        println!("l_no: {} -> new_idx: {}", l_no, new_idx);
                        let to_remove = map.remove(&(l_no as u32));
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
            if replaced_content_line_numbers.contains(&s_line_no) {
                // This line was replaced and not deleted -> and then added.
                line_detail[0].commit_hashes.push(commit_hash.clone());
            } else {
                line_detail[0].commit_hashes = vec![commit_hash];
            }
            let new_content = line_change_after
                .changed_content
                .get((s_line_no - line_change_after.start_line_number) as usize)
                .unwrap()
                .to_string();
            line_detail[0].content = new_content;
        }
        Some(DiffCases::NewLinesAdded) => {
            // Handle this case
            let s_line_no = line_change_after.start_line_number;
            let e_line_no = line_change_after.start_line_number + line_change_after.change_count;

            // Anything from s_line_no until the end should be right moved by the diff
            // diff is line_change_after.change_count.
            let mut to_remove_map: HashMap<u32, Vec<LineDetail>> = HashMap::new();
            for l_no in map.keys().cloned().collect::<Vec<u32>>() {
                if l_no >= s_line_no {
                    let new_idx = l_no + line_change_after.change_count;
                    let to_remove = map.remove(&l_no);
                    if to_remove.is_none() {
                        panic!("Line number {} not found in map", l_no);
                    }
                    println!("Moving line: {} to new line: {}", l_no, new_idx);
                    to_remove_map.insert(new_idx, to_remove.unwrap());
                }
            }
            // Now insert the new entries.
            for (l_no, line_detail) in to_remove_map {
                println!("Inserting line: {}", l_no);
                map.insert(l_no, line_detail);
            }
            // Now add the new lines.
            for l_no in s_line_no..e_line_no {
                println!("Adding line: {}", l_no);
                let new_content = line_change_after
                    .changed_content
                    .get((l_no - s_line_no) as usize)
                    .unwrap()
                    .to_string();
                map.remove(&l_no);
                map.insert(
                    l_no,
                    vec![LineDetail {
                        content: new_content,
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

pub fn fetch_line_numbers(line: String) -> (LineChange, LineChange) {
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
    }
    Ok(())
}

fn extract_commit_hashes(commit_hash: &str) {
    // Call git show --unified=0 for the commit_hash and extract line->[commit_hash...] list.
    let output = std::process::Command::new("git")
        .arg("show")
        .arg("--unified=0")
        .arg(commit_hash)
        .output()
        .expect("Failed to execute command");
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        // Pass the commit diff into
        let mut map: HashMap<u32, Vec<LineDetail>> = HashMap::new();
        let _ = parse_diff(commit_hash.to_string(), stdout, &mut map);
        println!("map length: {}", map.len());
    } else {
        eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
    }
}
