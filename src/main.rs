use std::fs::read;
use std::path::Path;
use std::path::PathBuf;

mod algo_loc;
mod authordetails_impl;
mod config;
mod contextgpt_structs;

use linecount::count_lines;
use quicli::prelude::*;
use structopt::StructOpt;

use algo_loc::get_unique_files_changed;
use contextgpt_structs::{Cli, CliAsync};

use crate::algo_loc::get_contextual_authors;

fn _validate_path(path: &String) -> bool {
    let path_obj = Path::new(path);
    path_obj.exists()
}

fn _validate_dir(path: &Path) -> bool {
    path.is_dir()
}

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

fn get_all_files(folder_path: &Path) -> Vec<PathBuf> {
    let mut output_files: Vec<PathBuf> = [].to_vec();
    let flattened_read_dir = folder_path
        .read_dir()
        .expect("Unable to read from given path")
        .flatten();
    for each_dir in flattened_read_dir {
        let possible_dir_path = each_dir.path();
        if possible_dir_path.is_dir() {
            // iterate through it again
            output_files.append(&mut get_all_files(&possible_dir_path));
        } else {
            output_files.push(possible_dir_path);
        }
    }
    output_files
}

// TODO: This function might not be needed but still thinking over it
// fn validate_strings_as_files(gitignore_data: &Vec<String>) {
//     for each_file in gitignore_data {
//         let path = Path::from(each_file);

//     }
// }

fn check_if_should_be_ignored(
    workspace_path: Option<&Path>,
    file_path_buf: &Path,
    gitignore_data: &Vec<String>,
) -> bool {
    let dir_path = match workspace_path {
        Some(val) => val.to_path_buf(),
        None => std::env::current_dir().unwrap(),
    };
    // println!("dirpath: {:?}", dir_path);

    let mut output_path = file_path_buf.canonicalize().unwrap();

    if dir_path.is_relative() {
        output_path = output_path
            .as_path()
            .strip_prefix(std::env::current_dir().unwrap())
            .expect("This shouldn't happen")
            .to_path_buf();
    } else {
        output_path = output_path
            .as_path()
            .strip_prefix(dir_path.clone())
            .unwrap_or(file_path_buf)
            .to_path_buf();
    }

    let mut ignore_file = false;
    for to_be_ignored in gitignore_data {
        if to_be_ignored.is_empty() {
            continue;
        }
        let new_str = to_be_ignored.strip_suffix('*').unwrap_or(to_be_ignored);

        if output_path.starts_with(new_str) {
            // TODO: Log here that file matched
            // TODO: make a map here to not do this comparison again by looping around?
            // println!(
            //     "stripped path: {:?}, and to be ignored: {:?}",
            //     output_path, new_str
            // );
            ignore_file = true;
            break;
        }
    }
    ignore_file
}

fn walk(workspace_path_buf: &Path, gitignore_data: &Vec<String>) {
    // TODO: Not sure if this function is needed right now
    // let validated_gitignore_files = validate_strings_as_files(gitignore_data);
    let all_files = get_all_files(workspace_path_buf);
    for each_file in all_files {
        let entry_path = each_file.as_path();

        let should_ignore =
            check_if_should_be_ignored(Some(workspace_path_buf), entry_path, gitignore_data);
        if should_ignore {
            continue;
        }
        if !_validate_dir(entry_path) {
            // It's a file!!
            println!("File: {:?}", entry_path);
            call_command_unique_files(entry_path);
        } else {
            // It's a directory
            walk(entry_path, gitignore_data);
        }
    }
}

fn read_gitignore(gitignore_file_path: String) -> Vec<String> {
    let gitignore_path = Path::new(&gitignore_file_path);
    let gitignore_data = read(gitignore_path).expect("Unable to read .gitignore file");
    let data = String::from_utf8(gitignore_data)
        .map_err(|non_utf8| String::from_utf8_lossy(non_utf8.as_bytes()).into_owned())
        .unwrap();
    let vec_data: Vec<String> = data.split('\n').map(String::from).collect();
    vec_data
}

// Run asynchronously in the bg
fn main() -> CliResult {
    // this runs for all files in the given workspace
    let args = CliAsync::from_args();
    if !_validate_path(&args.workspace_path) {
        // LOG ERROR``
        // return Err("Not a valid path you passed... hmm!");
    }
    let workspace_path = Path::new(&args.workspace_path);
    let current_folder_path: PathBuf = workspace_path.to_path_buf();
    if !_validate_dir(&current_folder_path) {
        /*
        TODO: Convert this into ExitFailure later
        Err("Not a valid directory you passed... hmm!")
        */
        return Ok(());
    }

    // TODO: Ideally, pass a function (something like func pointer) to walk along
    let gitignore_data: Vec<String> = read_gitignore(".gitignore".to_string());
    walk(
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

mod test;
