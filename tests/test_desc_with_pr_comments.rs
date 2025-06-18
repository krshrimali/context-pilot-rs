use contextpilot::contextgpt_structs::RequestTypeOptions;
use std::str::FromStr;

#[test]
fn test_desc_with_pr_comments_parsing() {
    // Test that the new mode can be parsed from strings
    assert_eq!(
        RequestTypeOptions::from_str("descwithprcomments").unwrap(),
        RequestTypeOptions::DescWithPRComments
    );

    assert_eq!(
        RequestTypeOptions::from_str("desc-with-pr-comments").unwrap(),
        RequestTypeOptions::DescWithPRComments
    );
}
