use std::fs::read;
use std::path::Path;
use std::path::PathBuf;

use crate::algo_loc::_correct_file_path;
use crate::call_command_unique_files;

pub fn validate_path(path: &String) -> bool {
    let path_obj = Path::new(path);
    path_obj.exists()
}

pub fn validate_dir(path: &Path) -> bool {
    path.is_dir()
}

pub fn get_all_files(folder_path: &Path) -> Vec<PathBuf> {
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

pub fn check_if_should_be_ignored(
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

pub fn walk(workspace_path_buf: &Path, gitignore_data: &Vec<String>) {
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
        if !validate_dir(entry_path) {
            // It's a file!!
            println!("File: {:?}", entry_path);
            call_command_unique_files(entry_path);
        } else {
            // It's a directory
            walk(entry_path, gitignore_data);
        }
    }
}

pub fn read_gitignore(gitignore_file_path: String) -> Vec<String> {
    let gitignore_path = Path::new(&gitignore_file_path);
    let gitignore_data = read(gitignore_path).expect("Unable to read .gitignore file");
    let data = String::from_utf8(gitignore_data)
        .map_err(|non_utf8| String::from_utf8_lossy(non_utf8.as_bytes()).into_owned())
        .unwrap();
    let vec_data: Vec<String> = data.split('\n').map(String::from).collect();
    vec_data
}

pub fn get_correct_file_path(original_file_path: &String) -> String {
    let path_obj = Path::new(original_file_path);
    if path_obj.is_absolute() && path_obj.exists() {
        "".to_string()
    } else {
        let output = _correct_file_path(path_obj);
        match output {
            Some(correct_path) => correct_path,
            None => "".to_string(),
        }
    }
}
