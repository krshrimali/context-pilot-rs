mod algo_loc;
mod authordetails_impl;
mod config;
mod contextgpt_structs;

use linecount::count_lines;
use quicli::prelude::*;
use structopt::StructOpt;

use algo_loc::get_unique_files_changed;
use contextgpt_structs::Cli;

use crate::algo_loc::get_contextual_authors;

// #[warn(dead_code)]
fn main() -> CliResult {
    let args = Cli::from_args();
    let end_line_number: usize = if args.end_number == 0 {
        0
    } else {
        args.end_number
    };
    let count_lines: i32 = count_lines(std::fs::File::open(args.file.clone()).unwrap())
        .unwrap()
        .try_into()
        .unwrap();
    let valid_end_line_number: usize = if args.end_number == 0 {
        (count_lines - 1).try_into().unwrap()
    } else {
        end_line_number
    };
    if args.request_type.starts_with("aut") {
        let output = get_contextual_authors(args.file, args.start_number, valid_end_line_number);
        println!("{:?}", output);
    } else {
        let output = get_unique_files_changed(args.file, args.start_number, valid_end_line_number);
        println!("{:?}", output);
    }
    Ok(())
}
