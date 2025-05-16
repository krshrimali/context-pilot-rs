use std::str::FromStr;

use serde::{Deserialize, Serialize};
use structopt::StructOpt;

// This also adds an impl: get_field to get the corresponding field from the field name (&str)
#[macro_export]
macro_rules! get_struct_names {
    (
        #[derive($($derive_name:ident),*)]
        pub enum $name:ident {
            $($fname:ident), *
        }
    ) => {
        #[derive($($derive_name),*)]
        pub enum $name {
            $($fname),*
        }

        impl $name {
            fn field_names() -> &'static [&'static str] {
                static NAMES: &'static [&'static str] = &[$(stringify!($fname)), *];
                NAMES
            }
        }
    }
}

get_struct_names! {
    #[derive(Debug, Eq, PartialEq, Clone)]
    pub enum RequestTypeOptions {
        Author,
        File,
        Index,
        Query,
        Descriptions  // alias: desc
    }
}

impl FromStr for RequestTypeOptions {
    type Err = String;
    fn from_str(request_type: &str) -> Result<Self, Self::Err> {
        match request_type {
            "author" => Ok(RequestTypeOptions::Author),
            "file" => Ok(RequestTypeOptions::File),
            "index" => Ok(RequestTypeOptions::Index),
            "query" => Ok(RequestTypeOptions::Query),
            "descriptions" => Ok(RequestTypeOptions::Descriptions),
            "desc" => Ok(RequestTypeOptions::Descriptions),
            _ => Err(format!(
                "Could not parse the request type: {}, available field names: {:?}",
                request_type,
                RequestTypeOptions::field_names()
            )),
        }
    }
}

#[derive(Debug, StructOpt)]
pub(crate) struct Cli {
    pub folder_path: String,
    pub file: Option<String>,

    #[structopt(short = "s")]
    pub start_number: Option<usize>,
    #[structopt(short = "e")]
    pub end_number: Option<usize>,

    // TODO: Add instructions on what request_type could be
    #[structopt(short = "t")]
    pub request_type: RequestTypeOptions,

    #[structopt(short = "i")]
    pub index_subfolder: Option<String>,
}

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct AuthorDetails {
    pub commit_hash: String,
    pub author_full_name: String,
    pub origin_file_path: String,
    pub contextual_file_paths: Vec<String>,
    pub line_number: usize,
    pub end_line_number: usize,
}

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct AuthorDetailsV2 {
    pub line_number: usize,
    pub origin_file_path: String,
    pub commit_hashes: Vec<String>,
    pub author_full_name: Vec<String>,
}
