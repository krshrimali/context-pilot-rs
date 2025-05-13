use contextpilot::diff::{extract_details_with_tracking, shift_lines_after_insert};
use std::collections::HashMap; // Adjust path if needed

#[test]
fn test_multiple_commits_same_file_different_lines() {
    let file_path = "src/main.rs";
    let mut map = HashMap::new();

    // Commit 1 adds lines 1 and 2
    extract_details_with_tracking(
        "commit1",
        vec![
            "@@ -0,0 +1,2 @@".to_string(),
            "+let a = 1;".to_string(),
            "+let b = 2;".to_string(),
        ],
        file_path,
        &mut map,
    );

    // Commit 2 edits line 1
    extract_details_with_tracking(
        "commit2",
        vec![
            "@@ -1,1 +1,1 @@".to_string(),
            "-let a = 1;".to_string(),
            "+let a = 10;".to_string(),
        ],
        file_path,
        &mut map,
    );

    // Commit 3 edits line 2
    extract_details_with_tracking(
        "commit3",
        vec![
            "@@ -2,1 +2,1 @@".to_string(),
            "-let b = 2;".to_string(),
            "+let b = 20;".to_string(),
        ],
        file_path,
        &mut map,
    );

    let file_lines = map.get(file_path).unwrap();

    let (content1, history1) = file_lines.get(&1).unwrap();
    assert_eq!(content1, "let a = 10;");
    assert_eq!(
        history1,
        &vec!["commit1".to_string(), "commit2".to_string()]
    );

    let (content2, history2) = file_lines.get(&2).unwrap();
    assert_eq!(content2, "let b = 20;");
    assert_eq!(
        history2,
        &vec!["commit1".to_string(), "commit3".to_string()]
    );
}

#[test]
fn test_add_and_edit_multiple_lines_in_one_commit() {
    let file_path = "src/compute.rs";
    let mut map = HashMap::new();

    extract_details_with_tracking(
        "commit_alpha",
        vec![
            "@@ -0,0 +1,3 @@".to_string(),
            "+fn compute() {".to_string(),
            "+    let x = 42;".to_string(),
            "+}".to_string(),
        ],
        file_path,
        &mut map,
    );

    extract_details_with_tracking(
        "commit_beta",
        vec![
            "@@ -2,1 +2,1 @@".to_string(),
            "-    let x = 42;".to_string(),
            "+    let x = 100;".to_string(),
        ],
        file_path,
        &mut map,
    );

    let file_lines = map.get(file_path).unwrap();

    let (line2_content, line2_history) = file_lines.get(&2).unwrap();
    assert_eq!(line2_content, "    let x = 100;");
    assert_eq!(
        line2_history,
        &vec!["commit_alpha".to_string(), "commit_beta".to_string()]
    );
}

#[test]
fn test_line_addition_and_later_move() {
    let file_path = "src/utils.rs";
    let mut map = HashMap::new();

    // Line added at line 1
    extract_details_with_tracking(
        "commit_first",
        vec!["@@ -0,0 +1,1 @@".to_string(), "+let temp = 99;".to_string()],
        file_path,
        &mut map,
    );

    // Simulate new lines added above, pushing the line down to line 3
    extract_details_with_tracking(
        "commit_second",
        vec![
            "@@ -0,0 +1,2 @@".to_string(),
            "+// header".to_string(),
            "+// another line".to_string(),
        ],
        file_path,
        &mut map,
    );

    // Track manually — this isn't auto remapping — test will show we haven't remapped
    assert!(map.get(file_path).unwrap().get(&3).is_none()); // line 3 not tracked unless remapping done
    assert!(map.get(file_path).unwrap().get(&1).is_some()); // old line still mapped at line 1
}

#[test]
fn test_edit_same_line_twice_different_commits() {
    let file_path = "src/edit_cycle.rs";
    let mut map = HashMap::new();

    // Original
    extract_details_with_tracking(
        "c1",
        vec![
            "@@ -0,0 +1,1 @@".to_string(),
            "+let user = \"admin\";".to_string(),
        ],
        file_path,
        &mut map,
    );

    // Change to 'root'
    extract_details_with_tracking(
        "c2",
        vec![
            "@@ -1,1 +1,1 @@".to_string(),
            "-let user = \"admin\";".to_string(),
            "+let user = \"root\";".to_string(),
        ],
        file_path,
        &mut map,
    );

    // Change to 'guest'
    extract_details_with_tracking(
        "c3",
        vec![
            "@@ -1,1 +1,1 @@".to_string(),
            "-let user = \"root\";".to_string(),
            "+let user = \"guest\";".to_string(),
        ],
        file_path,
        &mut map,
    );

    let (content, history) = map.get(file_path).unwrap().get(&1).unwrap();
    assert_eq!(content, "let user = \"guest\";");
    assert_eq!(
        history,
        &vec!["c1".to_string(), "c2".to_string(), "c3".to_string()]
    );
}

#[test]
fn test_shift_lines_after_insert() {
    let file_path = "src/example.rs";
    let mut map = HashMap::new();

    map.insert(
        file_path.to_string(),
        HashMap::from([
            (10, ("line10".to_string(), vec!["a".to_string()])),
            (12, ("line12".to_string(), vec!["b".to_string()])),
            (15, ("line15".to_string(), vec!["c".to_string()])),
        ]),
    );

    shift_lines_after_insert(file_path, 12, 5, &mut map);

    let file_map = map.get(file_path).unwrap();
    assert!(file_map.get(&12).is_none()); // line 12 got shifted
    assert_eq!(file_map.get(&17).unwrap().0, "line12"); // 12 → 17
    assert_eq!(file_map.get(&20).unwrap().0, "line15"); // 15 → 20
}
