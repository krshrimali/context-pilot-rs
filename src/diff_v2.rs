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

            for l_no in s_line_no..= e_line_no {
                if let Some(line_details) = map.get_mut(&l_no) {
                    for line_detail in line_details.iter_mut() {
                        line_detail.commit_hashes.push("commit_hash".to_string());
                    }
                }
            }
        }
        Some(DiffCases::FewLinesReplacedWithFewLines) => {
            // Handle this case
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
            },
            LineChange {
                start_line_number: line_after,
                change_type: ChangeType::Added,
                change_count: line_after_count,
            },
        )
    } else {
        panic!("Invalid line format: {}", line);
    }
}

fn parse_diff(commit_diff: String, map: &mut HashMap<u32, Vec<LineDetail>>) -> Result<(), String> {
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
            reorder_map(category, map, line_before, line_after);
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

        let mut map: HashMap<u32, Vec<LineDetail>> = HashMap::new();
        parse_diff(commit_diff.to_string(), &mut map).unwrap();

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
        let mut map: HashMap<u32, Vec<LineDetail>> = HashMap::new();
        map.insert(
            1,
            vec![LineDetail {
                content: "line1".to_string(),
                commit_hashes: vec!["hash1".to_string()],
            }],
        );

        reorder_map(Some(DiffCases::FewLinesReplacedWithSingleLine), &mut map);
        // Add assertions based on expected behavior.
    }
}
