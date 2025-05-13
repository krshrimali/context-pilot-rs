pub const LAST_MANY_COMMIT_HASHES: usize = 5;
// pub const AUTHOR_DB_PATH: &str = "db_common.json";
// pub const FILE_DB_PATH: &str = "db_common.json";
pub const DB_FOLDER: &str = ".contextpilot_db";
pub const MAX_ITEMS_IN_EACH_DB_FILE: u32 = 30; // arbitrary number for each DB to split up after it crosses this limit

// Some more flags we'll need in the future
// pub const FILE_COUNT_THRESHOLD: usize = 5;

// Maybe I would prefer Lua as we go ahead
pub const CONFIG_FILE_NAME: &str = "contextpilot.json";

// Default for the output to be shown in UI for selector
// For both request types: file and author
pub const OUTPUT_COUNT_THRESHOLD: usize = 10;

// TODO: Implement threshold for confidence for relevance
