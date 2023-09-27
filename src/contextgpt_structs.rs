use serde::{Deserialize, Serialize};
use std::str::FromStr;
use structopt::StructOpt;

#[derive(Debug)]
pub enum RequestTypeOptions {
    Author,
    File,
}

impl FromStr for RequestTypeOptions {
    type Err = String;
    fn from_str(request_type: &str) -> Result<Self, Self::Err> {
        match request_type {
            "author" => Ok(RequestTypeOptions::Author),
            "file" => Ok(RequestTypeOptions::File),
            // TODO: Pass the request_type in this error
            _ => Err(format!(
                "Could not parse the request type: {}",
                request_type
            )),
        }
    }
}

#[derive(Debug, StructOpt)]
pub struct Cli {
    pub file: String,

    #[structopt(short = "s")]
    pub start_number: usize,
    #[structopt(short = "e")]
    pub end_number: usize,

    // TODO: Add instructions on what request_type could be
    // TODO: Conver this from String to an Enum
    #[structopt(short = "t")]
    pub request_type: RequestTypeOptions,
}

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct AuthorDetails {
    pub commit_hash: String,
    pub author_full_name: String,
    pub origin_file_path: String,
    pub contextual_file_paths: Vec<String>,
    pub line_number: usize,
}
