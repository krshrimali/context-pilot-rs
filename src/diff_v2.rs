use std::collections::HashMap;

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
    // changed_content: String,
    change_count: u32,
    change_type: ChangeType,
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

            // We only change from s_line_no+1 till the end_line_no, since s_line_no is essentially
            // replaced, so we need to start tracking for it again.
            for l_no in (s_line_no + 1)..=e_line_no {
                map.remove(&l_no);
            }

            // Now update all the lines in the hash map and shift them:
            for l_no in map.keys().cloned().collect::<Vec<u32>>() {
                if l_no > e_line_no {
                    let new_idx = l_no - (line_change_before.change_count - 1);
                    let to_remove = map.remove(&new_idx);
                    map.insert(new_idx, to_remove.unwrap());
                }
            }
            // For the first line that just got replaced, create a new entry.
            map.insert(
                s_line_no,
                vec![LineDetail {
                    content: "New Content".to_string(), // FIXME: We don't have content yet. This is bad?
                    commit_hashes: vec![commit_hash],
                }],
            );
        }
        Some(DiffCases::FewLinesReplacedWithFewLines) => {
            let s_line_no = line_change_before.start_line_number;
            let e_line_no = line_change_before.start_line_number + line_change_before.change_count;

            let diff =
                line_change_after.change_count as i32 - line_change_before.change_count as i32;
            if (diff > 0) {
                // We need to add lines.
                for l_no in s_line_no..=e_line_no {
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

                // Now, for e_line_no:e_line_no+diff, we need to move them first so that we don't
                // lose content.
                for l_no in (e_line_no + 1)..=(e_line_no + diff as u32) {
                    let new_idx = l_no + diff as u32;
                    let to_remove = map.remove(&new_idx);
                    map.insert(new_idx, to_remove.unwrap());

                    // Once this moving is done, we need to add lines for l_no:
                    map.insert(
                        l_no,
                        vec![LineDetail {
                            content: "New Content".to_string(), // FIXME: We don't have content yet. This is bad?
                            commit_hashes: vec![commit_hash.clone()],
                        }],
                    );
                }

                // We have handled e_line_no:e_line_no+diff; but anything after that, also needs to
                // be shifted.
                for l_no in map.keys().cloned().collect::<Vec<u32>>() {
                    if l_no > e_line_no + diff as u32 {
                        let new_idx = l_no + (diff as u32);
                        let to_remove = map.remove(&new_idx);
                        map.insert(new_idx, to_remove.unwrap());
                    }
                }
            } else {
                // Lines deleted > Lines added.
                for l_no in s_line_no..=e_line_no {
                    map.remove(&l_no);
                }

                // Add content for the new lines.
                for l_no in s_line_no
                    ..=(line_change_after.start_line_number + line_change_after.change_count)
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
            // Handle this case
        }
        Some(DiffCases::FewLinesDeleted) => {
            // Handle this case
        }
        Some(DiffCases::SingleLineReplacedWithAnotherSingleLine) => {
            // Handle this case
        }
        Some(DiffCases::NewLinesAdded) => {
            // Handle this case
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
                // changed_content: line.clone(),
            },
            LineChange {
                start_line_number: line_after,
                change_type: ChangeType::Added,
                change_count: line_after_count,
                // changed_content: line.clone(),
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
    for line in commit_diff.lines() {
        let line = line.trim();
        if line.starts_with("@@") {
            // Only process till the last '@@'
            let mut line = line.to_string();
            line = line.split_once("@@").unwrap().1.to_string();
            line = line.split_once("@@").unwrap().0.trim().to_string();
            let (line_before, line_after): (LineChange, LineChange) =
                fetch_line_numbers(line.clone());
            let category = categorize_diff(line.as_str());
            reorder_map(commit_hash.clone(), category, map, line_before, line_after);
        }
    }
    Ok(())
}

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

    #[test]
    fn test_full_commit_diff() {
        let commit_diff = r#"
            @@ -2,5 +2 @@
            -line1
            -line2
            +line3
            @@ -90,61 +86 @@
            -line4
            +line5
            @@ -2,5 +2,3 @@
            -line6
            +line7
            @@ -23,2 +18,7 @@
            -line8
            +line9
            @@ -45 +44,0 @@
            -line10
            @@ -50,3 +48,0 @@
            -line11
            @@ -159 +96 @@
            -line12
        "#;

        let commit_hash = "";

        let mut map: HashMap<u32, Vec<LineDetail>> = HashMap::new();
        parse_diff(commit_hash.to_string(), commit_diff.to_string(), &mut map).unwrap();

        assert_eq!(map.len(), 0); // Adjust based on your expectations
    }

    #[test]
    fn test_fetch_line_numbers_replace_few_lines_with_single_line() {
        let line = "-2,5 +2";
        let (line_before, line_after) = fetch_line_numbers(line.to_string());
        assert_eq!(
            line_before,
            LineChange {
                start_line_number: 2,
                change_count: 5,
                change_type: ChangeType::Deleted
            }
        );
        assert_eq!(
            line_after,
            LineChange {
                start_line_number: 2,
                change_count: 1,
                change_type: ChangeType::Added
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
                change_type: ChangeType::Deleted
            }
        );
        assert_eq!(
            line_after,
            LineChange {
                start_line_number: 2,
                change_count: 3,
                change_type: ChangeType::Added
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
                change_type: ChangeType::Deleted
            }
        );
        assert_eq!(
            line_after,
            LineChange {
                start_line_number: 44,
                change_count: 0,
                change_type: ChangeType::Added
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
                change_type: ChangeType::Deleted
            }
        );
        assert_eq!(
            line_after,
            LineChange {
                start_line_number: 48,
                change_count: 0,
                change_type: ChangeType::Added
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
                change_type: ChangeType::Deleted
            }
        );
        assert_eq!(
            line_after,
            LineChange {
                start_line_number: 96,
                change_count: 1,
                change_type: ChangeType::Added
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
                change_type: ChangeType::Deleted
            }
        );
        assert_eq!(
            line_after,
            LineChange {
                start_line_number: 104,
                change_count: 3,
                change_type: ChangeType::Added
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
    fn test_reorder_map() {
        // Test that the map is correctly reordered for the case below:
        // -2,5 +2,7
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
        reorder_map(
            "commit2".to_string(),
            Some(DiffCases::FewLinesReplacedWithFewLines),
            &mut map,
            LineChange {
                start_line_number: 2,
                change_count: 5,
                change_type: ChangeType::Deleted,
            },
            LineChange {
                start_line_number: 2,
                change_count: 7,
                change_type: ChangeType::Added,
            },
        );

        // Make sure that map keys are correct.
        assert_eq!(map.len(), 10);
        // Make sure that for 1st line number, the content is unchanged.
        assert_eq!(
            map.get(&1).unwrap()[0].content,
            "line1".to_string()
        );

        // Make sure that from 2 to 8 (inclusive), the content's commit hash is now commit2:
        for i in 2..=8 {
            assert_eq!(
                map.get(&i).unwrap()[0].commit_hashes[0],
                "commit2".to_string()
            );
        }

        // From 9 to 10 (inclusive), the content is the same as previous map, that is commit hashes
        // are commit1.
        for i in 9..=10 {
            assert_eq!(
                map.get(&i).unwrap()[0].content,
                "line".to_string() + &i.to_string()
            );
            assert_eq!(
                map.get(&i).unwrap()[0].commit_hashes[0],
                "commit1".to_string()
            );
        }
    }
}
