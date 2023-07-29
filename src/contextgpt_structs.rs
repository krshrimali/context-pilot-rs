use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Cli {
    pub file: String,
    pub workspace_path: String,

    #[structopt(short = "s")]
    pub start_number: usize,
    #[structopt(short = "e")]
    pub end_number: usize,

    #[structopt(short = "t")]
    pub request_type: String,
}

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct AuthorDetails {
    pub commit_hash: String,
    pub author_full_name: String,
    pub origin_file_path: String,
    pub contextual_file_paths: Vec<String>,
    pub line_number: usize,
}
