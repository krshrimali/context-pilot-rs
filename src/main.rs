use std::path::Path;
use std::path::PathBuf;

mod algo_loc;
mod authordetails_impl;
mod config;
mod contextgpt_structs;
mod file_utils;
mod test;

use linecount::count_lines;
use quicli::prelude::*;
use structopt::StructOpt;

use algo_loc::get_unique_files_changed;
use contextgpt_structs::{Cli, CliAsync};

use crate::algo_loc::get_contextual_authors;

fn call_command_unique_files(file_path: &Path) {
    let count_lines: i32 = count_lines(std::fs::File::open(file_path).unwrap())
        .unwrap()
        .try_into()
        .unwrap();
    let valid_end_line_number: usize = (count_lines).try_into().unwrap_or(1);
    // println!("Valid end line number: {:?}", valid_end_line_number);
    println!(
        "Relevant files: {:?}",
        get_unique_files_changed(
            file_path.to_str().unwrap().to_string(),
            1,
            valid_end_line_number
        )
    );
}

// Run asynchronously in the bg
fn main() -> CliResult {
    // this runs for all files in the given workspace
    let args = CliAsync::from_args();
    if !file_utils::validate_path(&args.workspace_path) {
        // LOG ERROR``
        // return Err("Not a valid path you passed... hmm!");
    }
    let workspace_path = Path::new(&args.workspace_path);
    let current_folder_path: PathBuf = workspace_path.to_path_buf();
    if !file_utils::validate_dir(&current_folder_path) {
        /*
        TODO: Convert this into ExitFailure later
        Err("Not a valid directory you passed... hmm!")
        */
        return Ok(());
    }

    // TODO: Ideally, pass a function (something like func pointer) to walk along
    let gitignore_data: Vec<String> = file_utils::read_gitignore(".gitignore".to_string());
    file_utils::walk(
        &current_folder_path,
        /*gitignore_data=*/ &gitignore_data,
    );

    Ok(())
}

#[warn(dead_code)]
fn main_sync() -> CliResult {
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
