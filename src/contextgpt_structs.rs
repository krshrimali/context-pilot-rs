use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Cli {
    pub file: String,

    #[structopt(short = "s")]
    pub start_number: usize,
    #[structopt(short = "e")]
    pub end_number: usize,

    #[structopt(short = "t")]
    pub request_type: String,
}

#[derive(Default, Debug, Clone)]
pub struct AuthorDetails {
    pub commit_hash: String,
    pub author_full_name: String,
    pub file_path: String,
    pub line_number: usize,
}
