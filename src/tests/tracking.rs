use std::collections::HashMap;

use diff::{extract_details_with_tracking, levenshtein}; // Adjust module path as needed

#[test]
fn test_levenshtein_basic_edit() {
    let a = "let a = 5;";
    let b = "let a = 6;";
    assert_eq!(levenshtein(a, b), 1);
}

#[test]
fn test_track_simple_addition() {
    let commit_hash = "abc123";
    let file_path = "src/main.rs";

    let diff = vec![
        "@@ -0,0 +1,1 @@".to_string(),
        "+let a = 10;".to_string(),
    ];

    let mut map = HashMap::new();
    extract_details_with_tracking(commit_hash, diff, file_path, &mut map);

    let entry = map.get(file_path).unwrap();
    let (content, history) = entry.get(&1).unwrap();
    assert_eq!(content, "let a = 10;");
    assert_eq!(history, &vec!["abc123".to_string()]);
}

#[test]
fn test_track_edit_within_threshold() {
    let commit_hash = "abc124";
    let file_path = "src/main.rs";

    let diff = vec![
        "@@ -2,1 +2,1 @@".to_string(),
        "-let a = 5;".to_string(),
        "+let a = 6;".to_string(),
    ];

    let mut map = HashMap::new();
    extract_details_with_tracking(commit_hash, diff, file_path, &mut map);

    let entry = map.get(file_path).unwrap();
    let (content, history) = entry.get(&2).unwrap();
    assert_eq!(content, "let a = 6;");
    assert_eq!(history, &vec!["abc124".to_string()]);
}

#[test]
fn test_ignore_deletion_not_matched() {
    let commit_hash = "abc125";
    let file_path = "src/main.rs";

    let diff = vec![
        "@@ -3,1 +3,0 @@".to_string(),
        "-this line was deleted".to_string(),
    ];

    let mut map = HashMap::new();
    extract_details_with_tracking(commit_hash, diff, file_path, &mut map);

    // Should be empty since deletion was unmatched
    assert!(map.get(file_path).is_none());
}

#[test]
fn test_multiple_commit_history_for_line() {
    let file_path = "src/main.rs";

    let mut map = HashMap::new();

    // First commit adds line
    extract_details_with_tracking(
        "commit1",
        vec![
            "@@ -0,0 +1,1 @@".to_string(),
            "+let version = 1;".to_string(),
        ],
        file_path,
        &mut map,
    );

    // Second commit edits that line
    extract_details_with_tracking(
        "commit2",
        vec![
            "@@ -1,1 +1,1 @@".to_string(),
            "-let version = 1;".to_string(),
            "+let version = 2;".to_string(),
        ],
        file_path,
        &mut map,
    );

    let entry = map.get(file_path).unwrap();
    let (content, history) = entry.get(&1).unwrap();
    assert_eq!(content, "let version = 2;");
    assert_eq!(history, &vec!["commit1".to_string(), "commit2".to_string()]);
}

#[test]
fn test_unmatched_addition_after_deletion() {
    let file_path = "src/main.rs";
    let mut map = HashMap::new();

    let diff = vec![
        "@@ -5,1 +5,1 @@".to_string(),
        "-fn main() {}".to_string(),
        "+fn start() {}".to_string(), // Different enough to be unmatched
    ];

    extract_details_with_tracking("commit3", diff, file_path, &mut map);

    let entry = map.get(file_path).unwrap();
    let (content, history) = entry.get(&5).unwrap();
    assert_eq!(content, "fn start() {}");
    assert_eq!(history, &vec!["commit3".to_string()]);
}
