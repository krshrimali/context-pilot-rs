// mod algo_loc;
mod authordetails_impl;
mod config;
mod contextgpt_structs;
// mod db;
mod db_new;
mod git_command_algo;

use linecount::count_lines;
use quicli::prelude::*;
use structopt::StructOpt;

// use algo_loc::get_unique_files_changed;
use contextgpt_structs::{Cli, RequestTypeOptions};

// use crate::algo_loc::get_contextual_authors;

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
    // let mut db_obj = db_new::DB {
    //     db_file_name: config::AUTHOR_DB_PATH.to_string(),
    //     ..Default::default()
    // };
    // let mut db_obj = db::DB {
    //     db_file_name: config::FILE_DB_PATH.to_string(),
    //     ..Default::default()
    // };
    let mut db_obj = db_new::DB {
        folder_path: args.folder_path,
        ..Default::default()
    };
    match args.request_type {
        RequestTypeOptions::File => {
            db_obj.init_db(args.file.as_str());
            println!("Initialised for File");
            // let output = get_unique_files_changed(
            //     args.file,
            //     &args.start_number,
            //     &valid_end_line_number,
            //     &mut db_obj,
            // );
            // println!("{:?}", output);
        }
        RequestTypeOptions::Author => {
            db_obj.init_db(args.file.as_str());
            println!("Initialised for author");
            // let output = get_contextual_authors(
            //     args.file,
            //     &args.start_number,
            //     &valid_end_line_number,
            //     &mut db_obj,
            // );
            // println!("{:?}", output);
        }
    };
    Ok(())
}

