use contextpilot::diff_v2::{
    categorize_diff, fetch_line_numbers, reorder_map, ChangeType, DiffCases, LineChange, LineDetail,
};
use std::collections::HashMap;

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
        let line_number_categories = fetch_line_numbers(line.to_string());
        let (line_before, line_after) = line_number_categories.unwrap();
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
        let line_number_categories = fetch_line_numbers(line.to_string());
        let (line_before, line_after) = line_number_categories.unwrap();
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
        let line_number_categories = fetch_line_numbers(line.to_string());
        let (line_before, line_after) = line_number_categories.unwrap();
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
        let line_number_categories = fetch_line_numbers(line.to_string());
        let (line_before, line_after) = line_number_categories.unwrap();
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
        let line_number_categories = fetch_line_numbers(line.to_string());
        let (line_before, line_after) = line_number_categories.unwrap();
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
        let line_number_categories = fetch_line_numbers(line.to_string());
        let (line_before, line_after) = line_number_categories.unwrap();
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
                "commit1".to_string()
            );
        }
        assert_eq!(map.get(&2).unwrap()[0].content, "line23".to_string()); // New content.
        assert_eq!(map.get(&3).unwrap()[0].content, "line7".to_string());
        assert_eq!(map.get(&4).unwrap()[0].content, "line8".to_string());
        assert_eq!(map.get(&5).unwrap()[0].content, "line9".to_string());
    }

    #[test]
    fn test_reorder_map_few_lines_replaced_with_more_lines() {
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
        // Make sure that from 3 to 5 (inclusive), the content's commit hash is still commit1:
        for i in 3..=5 {
            assert_eq!(
                map.get(&i).unwrap()[0].commit_hashes[0],
                "commit1".to_string()
            );
        }
        assert_eq!(map.get(&2).unwrap()[0].content, "line23".to_string()); // New content.
        assert_eq!(map.get(&3).unwrap()[0].content, "line7".to_string());
        assert_eq!(map.get(&4).unwrap()[0].content, "line8".to_string());
        assert_eq!(map.get(&5).unwrap()[0].content, "line9".to_string());
        reorder_map(
            "commit3".to_string(),
            Some(DiffCases::FewLinesReplacedWithFewLines),
            &mut map,
            LineChange {
                start_line_number: 7,
                change_count: 2,
                change_type: ChangeType::Deleted,
                changed_content: vec!["line7".to_string(), "line7".to_string()],
            },
            LineChange {
                start_line_number: 3,
                change_count: 4,
                change_type: ChangeType::Added,
                changed_content: vec![
                    "line7".to_string(), // Same! - replaced
                    "line8".to_string(), // Same! - replaced
                    "new_content".to_string(),
                    "new_content".to_string(),
                ],
            },
            vec![3, 4],
        );
        assert_eq!(map.len(), 7);
        // Two lines were same, 3rd and 4th (after commit).
        assert_eq!(map.get(&3).unwrap()[0].content, "line7".to_string());
        assert_eq!(
            map.get(&3).unwrap()[0].commit_hashes,
            ["commit1".to_string(), "commit3".to_string()]
        );
        assert_eq!(map.get(&4).unwrap()[0].content, "line8".to_string());
        assert_eq!(
            map.get(&4).unwrap()[0].commit_hashes,
            ["commit1".to_string(), "commit3".to_string()]
        );
        // 5th and 6th lines are totally new, so commit hash should just be commit3.
        assert_eq!(map.get(&5).unwrap()[0].content, "new_content".to_string());
        assert_eq!(
            map.get(&5).unwrap()[0].commit_hashes,
            ["commit3".to_string()]
        );
        assert_eq!(map.get(&6).unwrap()[0].content, "new_content".to_string());
        assert_eq!(
            map.get(&6).unwrap()[0].commit_hashes,
            ["commit3".to_string()]
        );
        // Nothing happened to this line, so just make sure 7th here matches the 9th line
        // before this change.
        assert_eq!(map.get(&7).unwrap()[0].content, "line9".to_string());
        assert_eq!(
            map.get(&7).unwrap()[0].commit_hashes,
            ["commit1".to_string()]
        );
    }

    #[test]
    fn test_reorder_map_few_lines_replaced_with_less_lines() {
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
        // Make sure that from 3 to 5 (inclusive), the content's commit hash is still commit1:
        for i in 3..=5 {
            assert_eq!(
                map.get(&i).unwrap()[0].commit_hashes[0],
                "commit1".to_string()
            );
        }
        assert_eq!(map.get(&2).unwrap()[0].content, "line23".to_string()); // New content.
        assert_eq!(map.get(&3).unwrap()[0].content, "line7".to_string());
        assert_eq!(map.get(&4).unwrap()[0].content, "line8".to_string());
        assert_eq!(map.get(&5).unwrap()[0].content, "line9".to_string());
        reorder_map(
            "commit3".to_string(),
            Some(DiffCases::FewLinesReplacedWithFewLines),
            &mut map,
            LineChange {
                start_line_number: 7,
                change_count: 2,
                change_type: ChangeType::Deleted,
                changed_content: vec!["line7".to_string(), "line7".to_string()],
            },
            LineChange {
                start_line_number: 3,
                change_count: 1,
                change_type: ChangeType::Added,
                changed_content: vec![
                    "line7".to_string(), // Same! - replaced
                ],
            },
            vec![3],
        );
        assert_eq!(map.len(), 4);
        // 3rd line should be the same.
        assert_eq!(map.get(&3).unwrap()[0].content, "line7".to_string());
        assert_eq!(
            map.get(&3).unwrap()[0].commit_hashes,
            ["commit1".to_string(), "commit3".to_string()]
        );
        assert_eq!(map.get(&4).unwrap()[0].content, "line9".to_string());
        assert_eq!(
            map.get(&4).unwrap()[0].commit_hashes,
            ["commit1".to_string()]
        );
    }

    #[test]
    fn test_reorder_map_new_lines_added_empty_map() {
        // The first commit will mostly always be "new lines added"
        // category.
        let mut map: HashMap<u32, Vec<LineDetail>> = HashMap::new();
        reorder_map(
            "commit1".to_string(),
            Some(DiffCases::NewLinesAdded),
            &mut map,
            LineChange {
                start_line_number: 1,
                change_count: 0,
                change_type: ChangeType::Deleted,
                changed_content: vec![],
            },
            LineChange {
                start_line_number: 1,
                change_count: 2,
                change_type: ChangeType::Added,
                changed_content: vec!["line1".to_string(), "line2".to_string()],
            },
            vec![],
        );
        assert_eq!(map.len(), 2);
        assert_eq!(map.get(&1).unwrap()[0].content, "line1".to_string());
        assert_eq!(
            map.get(&1).unwrap()[0].commit_hashes,
            vec!["commit1".to_string()]
        );
        assert_eq!(map.get(&2).unwrap()[0].content, "line2".to_string());
        assert_eq!(
            map.get(&2).unwrap()[0].commit_hashes,
            vec!["commit1".to_string()]
        );
    }

    #[test]
    fn test_reorder_map_new_lines_added_nonempty_map() {
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
        // Make sure that from 3 to 5 (inclusive), the content's commit hash is still commit1:
        for i in 3..=5 {
            assert_eq!(
                map.get(&i).unwrap()[0].commit_hashes[0],
                "commit1".to_string()
            );
        }
        assert_eq!(map.get(&2).unwrap()[0].content, "line23".to_string()); // New content.
        assert_eq!(map.get(&3).unwrap()[0].content, "line7".to_string());
        assert_eq!(map.get(&4).unwrap()[0].content, "line8".to_string());
        assert_eq!(map.get(&5).unwrap()[0].content, "line9".to_string());
        reorder_map(
            "commit3".to_string(),
            Some(DiffCases::FewLinesReplacedWithFewLines),
            &mut map,
            LineChange {
                start_line_number: 7,
                change_count: 2,
                change_type: ChangeType::Deleted,
                changed_content: vec!["line7".to_string(), "line7".to_string()],
            },
            LineChange {
                start_line_number: 3,
                change_count: 4,
                change_type: ChangeType::Added,
                changed_content: vec![
                    "line7".to_string(), // Same! - replaced
                    "line8".to_string(), // Same! - replaced
                    "new_content".to_string(),
                    "new_content".to_string(),
                ],
            },
            vec![3, 4],
        );
        assert_eq!(map.len(), 7);
        // Two lines were same, 3rd and 4th (after commit).
        assert_eq!(map.get(&3).unwrap()[0].content, "line7".to_string());
        assert_eq!(
            map.get(&3).unwrap()[0].commit_hashes,
            ["commit1".to_string(), "commit3".to_string()]
        );
        assert_eq!(map.get(&4).unwrap()[0].content, "line8".to_string());
        assert_eq!(
            map.get(&4).unwrap()[0].commit_hashes,
            ["commit1".to_string(), "commit3".to_string()]
        );
        // 5th and 6th lines are totally new, so commit hash should just be commit3.
        assert_eq!(map.get(&5).unwrap()[0].content, "new_content".to_string());
        assert_eq!(
            map.get(&5).unwrap()[0].commit_hashes,
            ["commit3".to_string()]
        );
        assert_eq!(map.get(&6).unwrap()[0].content, "new_content".to_string());
        assert_eq!(
            map.get(&6).unwrap()[0].commit_hashes,
            ["commit3".to_string()]
        );
        // Nothing happened to this line, so just make sure 7th here matches the 9th line
        // before this change.
        assert_eq!(map.get(&7).unwrap()[0].content, "line9".to_string());
        assert_eq!(
            map.get(&7).unwrap()[0].commit_hashes,
            ["commit1".to_string()]
        );
        // Test for new lines added in b/w:
        reorder_map(
            "commit5".to_string(),
            Some(DiffCases::NewLinesAdded),
            &mut map,
            LineChange {
                start_line_number: 7,
                change_count: 0,
                change_type: ChangeType::Deleted,
                changed_content: vec![],
            },
            LineChange {
                start_line_number: 3,
                change_count: 2,
                change_type: ChangeType::Added,
                changed_content: vec!["absolutely_new".to_string(), "okay new".to_string()],
            },
            vec![],
        );
        assert_eq!(map.len(), 9);
        assert_eq!(
            map.get(&3).unwrap()[0].content,
            "absolutely_new".to_string()
        );
        assert_eq!(
            map.get(&3).unwrap()[0].commit_hashes,
            ["commit5".to_string()]
        );
        assert_eq!(map.get(&4).unwrap()[0].content, "okay new".to_string());
        assert_eq!(
            map.get(&4).unwrap()[0].commit_hashes,
            ["commit5".to_string()]
        );
        for i in 5..=6 {
            assert_eq!(
                map.get(&i).unwrap()[0].commit_hashes,
                vec!["commit1".to_string(), "commit3".to_string()]
            );
        }
        for i in 7..=8 {
            assert_eq!(
                map.get(&i).unwrap()[0].commit_hashes,
                vec!["commit3".to_string()]
            );
        }
        assert_eq!(
            map.get(&9).unwrap()[0].commit_hashes,
            vec!["commit1".to_string()]
        );
    }

    #[test]
    fn test_reorder_map_single_line_replaced_with_another_single_line() {
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
        // Make sure that from 3 to 5 (inclusive), the content's commit hash is still commit1:
        for i in 3..=5 {
            assert_eq!(
                map.get(&i).unwrap()[0].commit_hashes[0],
                "commit1".to_string()
            );
        }
        assert_eq!(map.get(&2).unwrap()[0].content, "line23".to_string()); // New content.
        assert_eq!(map.get(&3).unwrap()[0].content, "line7".to_string());
        assert_eq!(map.get(&4).unwrap()[0].content, "line8".to_string());
        assert_eq!(map.get(&5).unwrap()[0].content, "line9".to_string());
        reorder_map(
            "commit3".to_string(),
            Some(DiffCases::FewLinesReplacedWithFewLines),
            &mut map,
            LineChange {
                start_line_number: 7,
                change_count: 2,
                change_type: ChangeType::Deleted,
                changed_content: vec!["line7".to_string(), "line7".to_string()],
            },
            LineChange {
                start_line_number: 3,
                change_count: 4,
                change_type: ChangeType::Added,
                changed_content: vec![
                    "line7".to_string(), // Same! - replaced
                    "line8".to_string(), // Same! - replaced
                    "new_content".to_string(),
                    "new_content".to_string(),
                ],
            },
            vec![3, 4],
        );
        assert_eq!(map.len(), 7);
        // Two lines were same, 3rd and 4th (after commit).
        assert_eq!(map.get(&3).unwrap()[0].content, "line7".to_string());
        assert_eq!(
            map.get(&3).unwrap()[0].commit_hashes,
            ["commit1".to_string(), "commit3".to_string()]
        );
        assert_eq!(map.get(&4).unwrap()[0].content, "line8".to_string());
        assert_eq!(
            map.get(&4).unwrap()[0].commit_hashes,
            ["commit1".to_string(), "commit3".to_string()]
        );
        // 5th and 6th lines are totally new, so commit hash should just be commit3.
        assert_eq!(map.get(&5).unwrap()[0].content, "new_content".to_string());
        assert_eq!(
            map.get(&5).unwrap()[0].commit_hashes,
            ["commit3".to_string()]
        );
        assert_eq!(map.get(&6).unwrap()[0].content, "new_content".to_string());
        assert_eq!(
            map.get(&6).unwrap()[0].commit_hashes,
            ["commit3".to_string()]
        );
        // Nothing happened to this line, so just make sure 7th here matches the 9th line
        // before this change.
        assert_eq!(map.get(&7).unwrap()[0].content, "line9".to_string());
        assert_eq!(
            map.get(&7).unwrap()[0].commit_hashes,
            ["commit1".to_string()]
        );
        // Test for new lines added in b/w:
        reorder_map(
            "commit5".to_string(),
            Some(DiffCases::NewLinesAdded),
            &mut map,
            LineChange {
                start_line_number: 7,
                change_count: 0,
                change_type: ChangeType::Deleted,
                changed_content: vec![],
            },
            LineChange {
                start_line_number: 3,
                change_count: 2,
                change_type: ChangeType::Added,
                changed_content: vec!["absolutely_new".to_string(), "okay new".to_string()],
            },
            vec![],
        );
        assert_eq!(map.len(), 9);
        assert_eq!(
            map.get(&3).unwrap()[0].content,
            "absolutely_new".to_string()
        );
        assert_eq!(
            map.get(&3).unwrap()[0].commit_hashes,
            ["commit5".to_string()]
        );
        assert_eq!(map.get(&4).unwrap()[0].content, "okay new".to_string());
        assert_eq!(
            map.get(&4).unwrap()[0].commit_hashes,
            ["commit5".to_string()]
        );
        for i in 5..=6 {
            assert_eq!(
                map.get(&i).unwrap()[0].commit_hashes,
                vec!["commit1".to_string(), "commit3".to_string()]
            );
        }
        for i in 7..=8 {
            assert_eq!(
                map.get(&i).unwrap()[0].commit_hashes,
                vec!["commit3".to_string()]
            );
        }
        assert_eq!(
            map.get(&9).unwrap()[0].commit_hashes,
            vec!["commit1".to_string()]
        );
        reorder_map(
            "commit5".to_string(),
            Some(DiffCases::SingleLineReplacedWithAnotherSingleLine),
            &mut map,
            LineChange {
                start_line_number: 7, // This anyways does not matter, so I'm keeping it anything.
                change_count: 1,
                change_type: ChangeType::Deleted,
                changed_content: vec![],
            },
            LineChange {
                start_line_number: 5,
                change_count: 1,
                change_type: ChangeType::Added,
                changed_content: vec!["similar".to_string()],
            },
            vec![5], // The line is being replaced; means content is "similar"
        );
        assert_eq!(map.len(), 9); // Length shouldn't change.
        assert_eq!(map.get(&5).unwrap()[0].content, "similar".to_string());
        assert_eq!(
            map.get(&5).unwrap()[0].commit_hashes,
            vec![
                "commit1".to_string(),
                "commit3".to_string(),
                "commit5".to_string()
            ]
        );
        // Test for another single line deleted, but this time not replaced.
        reorder_map(
            "commit5".to_string(),
            Some(DiffCases::SingleLineReplacedWithAnotherSingleLine),
            &mut map,
            LineChange {
                start_line_number: 7, // This anyways does not matter, so I'm keeping it anything.
                change_count: 1,
                change_type: ChangeType::Deleted,
                changed_content: vec![],
            },
            LineChange {
                start_line_number: 5,
                change_count: 1,
                change_type: ChangeType::Added,
                changed_content: vec!["not similar".to_string()],
            },
            vec![],
        );
        assert_eq!(map.len(), 9); // Length shouldn't change.
        assert_eq!(map.get(&5).unwrap()[0].content, "not similar".to_string());
        assert_eq!(
            map.get(&5).unwrap()[0].commit_hashes,
            vec!["commit5".to_string()]
        );
    }

    #[test]
    fn test_reorder_map_with_few_lines_deleted() {
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
        // Make sure that from 3 to 5 (inclusive), the content's commit hash is still commit1:
        for i in 3..=5 {
            assert_eq!(
                map.get(&i).unwrap()[0].commit_hashes[0],
                "commit1".to_string()
            );
        }
        assert_eq!(map.get(&2).unwrap()[0].content, "line23".to_string()); // New content.
        assert_eq!(map.get(&3).unwrap()[0].content, "line7".to_string());
        assert_eq!(map.get(&4).unwrap()[0].content, "line8".to_string());
        assert_eq!(map.get(&5).unwrap()[0].content, "line9".to_string());
        reorder_map(
            "commit3".to_string(),
            Some(DiffCases::FewLinesReplacedWithFewLines),
            &mut map,
            LineChange {
                start_line_number: 7,
                change_count: 2,
                change_type: ChangeType::Deleted,
                changed_content: vec!["line7".to_string(), "line7".to_string()],
            },
            LineChange {
                start_line_number: 3,
                change_count: 4,
                change_type: ChangeType::Added,
                changed_content: vec![
                    "line7".to_string(), // Same! - replaced
                    "line8".to_string(), // Same! - replaced
                    "new_content".to_string(),
                    "new_content".to_string(),
                ],
            },
            vec![3, 4],
        );
        assert_eq!(map.len(), 7);
        // Two lines were same, 3rd and 4th (after commit).
        assert_eq!(map.get(&3).unwrap()[0].content, "line7".to_string());
        assert_eq!(
            map.get(&3).unwrap()[0].commit_hashes,
            ["commit1".to_string(), "commit3".to_string()]
        );
        assert_eq!(map.get(&4).unwrap()[0].content, "line8".to_string());
        assert_eq!(
            map.get(&4).unwrap()[0].commit_hashes,
            ["commit1".to_string(), "commit3".to_string()]
        );
        // 5th and 6th lines are totally new, so commit hash should just be commit3.
        assert_eq!(map.get(&5).unwrap()[0].content, "new_content".to_string());
        assert_eq!(
            map.get(&5).unwrap()[0].commit_hashes,
            ["commit3".to_string()]
        );
        assert_eq!(map.get(&6).unwrap()[0].content, "new_content".to_string());
        assert_eq!(
            map.get(&6).unwrap()[0].commit_hashes,
            ["commit3".to_string()]
        );
        // Nothing happened to this line, so just make sure 7th here matches the 9th line
        // before this change.
        assert_eq!(map.get(&7).unwrap()[0].content, "line9".to_string());
        assert_eq!(
            map.get(&7).unwrap()[0].commit_hashes,
            ["commit1".to_string()]
        );
        // Test for new lines added in b/w:
        reorder_map(
            "commit5".to_string(),
            Some(DiffCases::NewLinesAdded),
            &mut map,
            LineChange {
                start_line_number: 7,
                change_count: 0,
                change_type: ChangeType::Deleted,
                changed_content: vec![],
            },
            LineChange {
                start_line_number: 3,
                change_count: 2,
                change_type: ChangeType::Added,
                changed_content: vec!["absolutely_new".to_string(), "okay new".to_string()],
            },
            vec![],
        );
        assert_eq!(map.len(), 9);
        assert_eq!(
            map.get(&3).unwrap()[0].content,
            "absolutely_new".to_string()
        );
        assert_eq!(
            map.get(&3).unwrap()[0].commit_hashes,
            ["commit5".to_string()]
        );
        assert_eq!(map.get(&4).unwrap()[0].content, "okay new".to_string());
        assert_eq!(
            map.get(&4).unwrap()[0].commit_hashes,
            ["commit5".to_string()]
        );
        for i in 5..=6 {
            assert_eq!(
                map.get(&i).unwrap()[0].commit_hashes,
                vec!["commit1".to_string(), "commit3".to_string()]
            );
        }
        for i in 7..=8 {
            assert_eq!(
                map.get(&i).unwrap()[0].commit_hashes,
                vec!["commit3".to_string()]
            );
        }
        assert_eq!(
            map.get(&9).unwrap()[0].commit_hashes,
            vec!["commit1".to_string()]
        );
        reorder_map(
            "commit5".to_string(),
            Some(DiffCases::SingleLineReplacedWithAnotherSingleLine),
            &mut map,
            LineChange {
                start_line_number: 7, // This anyways does not matter, so I'm keeping it anything.
                change_count: 1,
                change_type: ChangeType::Deleted,
                changed_content: vec![],
            },
            LineChange {
                start_line_number: 5,
                change_count: 1,
                change_type: ChangeType::Added,
                changed_content: vec!["similar".to_string()],
            },
            vec![5], // The line is being replaced; means content is "similar"
        );
        assert_eq!(map.len(), 9); // Length shouldn't change.
        assert_eq!(map.get(&5).unwrap()[0].content, "similar".to_string());
        assert_eq!(
            map.get(&5).unwrap()[0].commit_hashes,
            vec![
                "commit1".to_string(),
                "commit3".to_string(),
                "commit5".to_string()
            ]
        );
        // Test for another single line deleted, but this time not replaced.
        reorder_map(
            "commit5".to_string(),
            Some(DiffCases::SingleLineReplacedWithAnotherSingleLine),
            &mut map,
            LineChange {
                start_line_number: 7, // This anyways does not matter, so I'm keeping it anything.
                change_count: 1,
                change_type: ChangeType::Deleted,
                changed_content: vec![],
            },
            LineChange {
                start_line_number: 5,
                change_count: 1,
                change_type: ChangeType::Added,
                changed_content: vec!["not similar".to_string()],
            },
            vec![],
        );
        assert_eq!(map.len(), 9); // Length shouldn't change.
        assert_eq!(map.get(&5).unwrap()[0].content, "not similar".to_string());
        assert_eq!(
            map.get(&5).unwrap()[0].commit_hashes,
            vec!["commit5".to_string()]
        );
        let old_vals = map.clone();
        // Delete 5th, 6th line and make sure data is shifted accordingly.
        reorder_map(
            "commit6".to_string(),
            Some(DiffCases::FewLinesDeleted),
            &mut map,
            LineChange {
                start_line_number: 7, // Does not matter in tests;
                change_count: 2,
                change_type: ChangeType::Deleted,
                changed_content: vec![],
            },
            LineChange {
                start_line_number: 3,
                change_count: 0,
                change_type: ChangeType::Added,
                changed_content: vec![],
            },
            vec![],
        );
        assert_eq!(map.len(), 7); // Length should be reduced by 2.
        assert_eq!(map.get(&3).unwrap()[0].content, "not similar".to_string());
        assert_eq!(
            map.get(&3).unwrap()[0].commit_hashes,
            vec!["commit5".to_string()]
        );
        // Compare with old_vals map we used above for rest of the lines:
        for i in 1..=2 {
            assert_eq!(
                map.get(&i).unwrap()[0].content,
                old_vals.get(&i).unwrap()[0].content
            );
            assert_eq!(
                map.get(&i).unwrap()[0].commit_hashes,
                old_vals.get(&i).unwrap()[0].commit_hashes
            );
        }
        for i in 4..=7 {
            assert_eq!(
                map.get(&i).unwrap()[0].content,
                old_vals.get(&(i + 2)).unwrap()[0].content
            );
            assert_eq!(
                map.get(&i).unwrap()[0].commit_hashes,
                old_vals.get(&(i + 2)).unwrap()[0].commit_hashes
            );
        }
    }

    #[test]
    fn test_reorder_map_with_single_line_deleted() {
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
        // Make sure that from 3 to 5 (inclusive), the content's commit hash is still commit1:
        for i in 3..=5 {
            assert_eq!(
                map.get(&i).unwrap()[0].commit_hashes[0],
                "commit1".to_string()
            );
        }
        assert_eq!(map.get(&2).unwrap()[0].content, "line23".to_string()); // New content.
        assert_eq!(map.get(&3).unwrap()[0].content, "line7".to_string());
        assert_eq!(map.get(&4).unwrap()[0].content, "line8".to_string());
        assert_eq!(map.get(&5).unwrap()[0].content, "line9".to_string());
        reorder_map(
            "commit3".to_string(),
            Some(DiffCases::FewLinesReplacedWithFewLines),
            &mut map,
            LineChange {
                start_line_number: 7,
                change_count: 2,
                change_type: ChangeType::Deleted,
                changed_content: vec!["line7".to_string(), "line7".to_string()],
            },
            LineChange {
                start_line_number: 3,
                change_count: 4,
                change_type: ChangeType::Added,
                changed_content: vec![
                    "line7".to_string(), // Same! - replaced
                    "line8".to_string(), // Same! - replaced
                    "new_content".to_string(),
                    "new_content".to_string(),
                ],
            },
            vec![3, 4],
        );
        assert_eq!(map.len(), 7);
        // Two lines were same, 3rd and 4th (after commit).
        assert_eq!(map.get(&3).unwrap()[0].content, "line7".to_string());
        assert_eq!(
            map.get(&3).unwrap()[0].commit_hashes,
            ["commit1".to_string(), "commit3".to_string()]
        );
        assert_eq!(map.get(&4).unwrap()[0].content, "line8".to_string());
        assert_eq!(
            map.get(&4).unwrap()[0].commit_hashes,
            ["commit1".to_string(), "commit3".to_string()]
        );
        // 5th and 6th lines are totally new, so commit hash should just be commit3.
        assert_eq!(map.get(&5).unwrap()[0].content, "new_content".to_string());
        assert_eq!(
            map.get(&5).unwrap()[0].commit_hashes,
            ["commit3".to_string()]
        );
        assert_eq!(map.get(&6).unwrap()[0].content, "new_content".to_string());
        assert_eq!(
            map.get(&6).unwrap()[0].commit_hashes,
            ["commit3".to_string()]
        );
        // Nothing happened to this line, so just make sure 7th here matches the 9th line
        // before this change.
        assert_eq!(map.get(&7).unwrap()[0].content, "line9".to_string());
        assert_eq!(
            map.get(&7).unwrap()[0].commit_hashes,
            ["commit1".to_string()]
        );
        // Test for new lines added in b/w:
        reorder_map(
            "commit5".to_string(),
            Some(DiffCases::NewLinesAdded),
            &mut map,
            LineChange {
                start_line_number: 7,
                change_count: 0,
                change_type: ChangeType::Deleted,
                changed_content: vec![],
            },
            LineChange {
                start_line_number: 3,
                change_count: 2,
                change_type: ChangeType::Added,
                changed_content: vec!["absolutely_new".to_string(), "okay new".to_string()],
            },
            vec![],
        );
        assert_eq!(map.len(), 9);
        assert_eq!(
            map.get(&3).unwrap()[0].content,
            "absolutely_new".to_string()
        );
        assert_eq!(
            map.get(&3).unwrap()[0].commit_hashes,
            ["commit5".to_string()]
        );
        assert_eq!(map.get(&4).unwrap()[0].content, "okay new".to_string());
        assert_eq!(
            map.get(&4).unwrap()[0].commit_hashes,
            ["commit5".to_string()]
        );
        for i in 5..=6 {
            assert_eq!(
                map.get(&i).unwrap()[0].commit_hashes,
                vec!["commit1".to_string(), "commit3".to_string()]
            );
        }
        for i in 7..=8 {
            assert_eq!(
                map.get(&i).unwrap()[0].commit_hashes,
                vec!["commit3".to_string()]
            );
        }
        assert_eq!(
            map.get(&9).unwrap()[0].commit_hashes,
            vec!["commit1".to_string()]
        );
        reorder_map(
            "commit5".to_string(),
            Some(DiffCases::SingleLineReplacedWithAnotherSingleLine),
            &mut map,
            LineChange {
                start_line_number: 7, // This anyways does not matter, so I'm keeping it anything.
                change_count: 1,
                change_type: ChangeType::Deleted,
                changed_content: vec![],
            },
            LineChange {
                start_line_number: 5,
                change_count: 1,
                change_type: ChangeType::Added,
                changed_content: vec!["similar".to_string()],
            },
            vec![5], // The line is being replaced; means content is "similar"
        );
        assert_eq!(map.len(), 9); // Length shouldn't change.
        assert_eq!(map.get(&5).unwrap()[0].content, "similar".to_string());
        assert_eq!(
            map.get(&5).unwrap()[0].commit_hashes,
            vec![
                "commit1".to_string(),
                "commit3".to_string(),
                "commit5".to_string()
            ]
        );
        // Test for another single line deleted, but this time not replaced.
        reorder_map(
            "commit5".to_string(),
            Some(DiffCases::SingleLineReplacedWithAnotherSingleLine),
            &mut map,
            LineChange {
                start_line_number: 7, // This anyways does not matter, so I'm keeping it anything.
                change_count: 1,
                change_type: ChangeType::Deleted,
                changed_content: vec![],
            },
            LineChange {
                start_line_number: 5,
                change_count: 1,
                change_type: ChangeType::Added,
                changed_content: vec!["not similar".to_string()],
            },
            vec![],
        );
        assert_eq!(map.len(), 9); // Length shouldn't change.
        assert_eq!(map.get(&5).unwrap()[0].content, "not similar".to_string());
        assert_eq!(
            map.get(&5).unwrap()[0].commit_hashes,
            vec!["commit5".to_string()]
        );
        let old_vals = map.clone();
        // Delete 5th, 6th line and make sure data is shifted accordingly.
        reorder_map(
            "commit6".to_string(),
            Some(DiffCases::FewLinesDeleted),
            &mut map,
            LineChange {
                start_line_number: 7, // Does not matter in tests;
                change_count: 2,
                change_type: ChangeType::Deleted,
                changed_content: vec![],
            },
            LineChange {
                start_line_number: 3,
                change_count: 0,
                change_type: ChangeType::Added,
                changed_content: vec![],
            },
            vec![],
        );
        assert_eq!(map.len(), 7); // Length should be reduced by 2.
        assert_eq!(map.get(&3).unwrap()[0].content, "not similar".to_string());
        assert_eq!(
            map.get(&3).unwrap()[0].commit_hashes,
            vec!["commit5".to_string()]
        );
        // Compare with old_vals map we used above for rest of the lines:
        for i in 1..=2 {
            assert_eq!(
                map.get(&i).unwrap()[0].content,
                old_vals.get(&i).unwrap()[0].content
            );
            assert_eq!(
                map.get(&i).unwrap()[0].commit_hashes,
                old_vals.get(&i).unwrap()[0].commit_hashes
            );
        }
        for i in 4..=7 {
            assert_eq!(
                map.get(&i).unwrap()[0].content,
                old_vals.get(&(i + 2)).unwrap()[0].content
            );
            assert_eq!(
                map.get(&i).unwrap()[0].commit_hashes,
                old_vals.get(&(i + 2)).unwrap()[0].commit_hashes
            );
        }
        let old_vals = map.clone();
        reorder_map(
            "commit7".to_string(),
            Some(DiffCases::SingleLineDeleted),
            &mut map,
            LineChange {
                start_line_number: 7, // Does not matter in tests;
                change_count: 1,
                change_type: ChangeType::Deleted,
                changed_content: vec![],
            },
            LineChange {
                start_line_number: 2,
                change_count: 0,
                change_type: ChangeType::Added,
                changed_content: vec![],
            },
            vec![],
        );
        assert_eq!(map.len(), 6); // Length should be reduced by 1.
        assert_eq!(map.get(&2).unwrap()[0].content, "not similar".to_string());
        assert_eq!(
            map.get(&2).unwrap()[0].commit_hashes,
            vec!["commit5".to_string()]
        );
        // Compare with old_vals map we used above for rest of the lines:
        for i in 1..2 {
            assert_eq!(
                map.get(&i).unwrap()[0].content,
                old_vals.get(&i).unwrap()[0].content
            );
            assert_eq!(
                map.get(&i).unwrap()[0].commit_hashes,
                old_vals.get(&i).unwrap()[0].commit_hashes
            );
        }
        for i in 3..=6 {
            assert_eq!(
                map.get(&i).unwrap()[0].content,
                old_vals.get(&(i + 1)).unwrap()[0].content
            );
            assert_eq!(
                map.get(&i).unwrap()[0].commit_hashes,
                old_vals.get(&(i + 1)).unwrap()[0].commit_hashes
            );
        }
    }
}
