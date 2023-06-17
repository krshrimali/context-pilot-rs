use std::fs::ReadDir;
use std::iter::Flatten;
use std::path::Path;
use std::path::PathBuf;

use linecount::count_lines;
use quicli::prelude::*;
use structopt::StructOpt;

mod algo_loc;
mod authordetails_impl;
mod config;
mod contextgpt_structs;

use algo_loc::get_unique_files_changed;
use contextgpt_structs::Cli;
use contextgpt_structs::CliAsync;

use crate::algo_loc::get_contextual_authors;

fn _validate_path(path: &String) -> bool {
    let path_obj = Path::new(path);
    path_obj.exists()
}

fn _validate_dir(path: &Path) -> bool {
    path.is_dir()
}

fn call_command_unique_files(file_path: &PathBuf) {
    let count_lines: i32 = count_lines(std::fs::File::open(file_path.clone()).unwrap())
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

fn walk(workspace_path_buf: &Path) {
    let all_files = get_all_files(workspace_path_buf);
    for each_file in workspace_path_buf
        .read_dir()
        .expect("Unable to read directory from path each_file (TODO)")
        .flatten()
    {
        let entry_path = each_file.path();
        if !_validate_dir(&entry_path) {
            // It's a file!!
            if workspace_path_buf.to_str().unwrap().contains("target") {
                continue;
            }
            println!("File: {:?}", entry_path);
            // println!(
            //     "Relevant files found: {:?}",
            //     get_unique_files_changed(entry_path.to_str().unwrap().to_string(), 1, 10)
            // );
            call_command_unique_files(&entry_path);
            // println!("Relevant files: {:?}", get_(entry_path);
        } else {
            // It's a directory
            walk(&entry_path);
        }
    }
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

    // for each_file in Path::new(&args.workspace_path)
    //     .read_dir()
    //     .expect("Couldn't open the directory, something went wrong...")
    // {
    // TODO: Ideally, pass a function (something like func pointer) to walk along
    walk(&current_folder_path);

    Ok(())
}

/// .
///
/// # Panics
///
/// Panics if .
///
/// # Errors
///
/// This function will return an error if .
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

mod test {
    use std::path::{Path, PathBuf};

    use crate::get_all_files;
    use std::fs::{File, ReadDir};
    use std::io::{Error, Write};

    fn create_sample_testing_dir() {
        let output_folder_path = Path::new(".test");
        if !output_folder_path.is_dir() {
            std::fs::create_dir(output_folder_path).expect("Unable to create a directory");
        }
        let test_content = "test_content".to_string();
        let total_files: usize = 6;

        for idx in 1..total_files {
            let file_path = output_folder_path.to_str().unwrap().to_owned()
                + "/test_path"
                + &idx.to_string()
                + ".txt";
            let mut output_file =
                File::create(file_path).expect("File wasn't successfully created");
            write!(output_file, "{}", test_content).expect("Unable to write content to the file");
        }
    }

    fn create_sample_testing_dir_recursive() {
        let output_folder_path = Path::new(".test_recursive");
        if !output_folder_path.is_dir() {
            std::fs::create_dir(output_folder_path).expect("Unable to create a directory");
        }
        let test_content = "test_content".to_string();
        let total_files: usize = 6;

        for idx in 1..total_files / 2 {
            let output_recursed_folder_path =
                output_folder_path.to_str().unwrap().to_owned() + "/" + &idx.to_string();
            if !Path::new(&output_recursed_folder_path).is_dir() {
                std::fs::create_dir(output_recursed_folder_path.clone())
                    .expect("Unable to create a directory");
            }
            for idx_recurse in total_files / 2..total_files {
                let file_path = output_recursed_folder_path.clone()
                    + "/test_path"
                    + &idx_recurse.to_string()
                    + ".txt";
                let mut output_file =
                    File::create(file_path).expect("File wasn't successfully created");
                write!(output_file, "{}", test_content)
                    .expect("Unable to write content to the file");
            }
        }
    }

    #[test]
    fn test_get_all_files_non_recursive() {
        create_sample_testing_dir();
        let mut all_files = get_all_files(Path::new(".test/"));
        all_files.sort();
        assert_eq!(
            all_files,
            vec![
                Path::new(".test/test_path1.txt").to_path_buf(),
                Path::new(".test/test_path2.txt").to_path_buf(),
                Path::new(".test/test_path3.txt").to_path_buf(),
                Path::new(".test/test_path4.txt").to_path_buf(),
                Path::new(".test/test_path5.txt").to_path_buf(),
            ]
        );
    }

    #[test]
    fn test_get_all_files_recursive() {
        create_sample_testing_dir_recursive();
        let mut all_files = get_all_files(Path::new(".test_recursive/"));
        all_files.sort();
        assert_eq!(
            all_files,
            vec![
                Path::new(".test_recursive/1/test_path3.txt").to_path_buf(),
                Path::new(".test_recursive/1/test_path4.txt").to_path_buf(),
                Path::new(".test_recursive/1/test_path5.txt").to_path_buf(),
                Path::new(".test_recursive/2/test_path3.txt").to_path_buf(),
                Path::new(".test_recursive/2/test_path4.txt").to_path_buf(),
                Path::new(".test_recursive/2/test_path5.txt").to_path_buf(),
            ]
        );
    }
}
