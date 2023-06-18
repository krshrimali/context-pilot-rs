#[cfg(test)]
mod tests {
    use crate::file_utils::{check_if_should_be_ignored, get_all_files, read_gitignore};
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;

    fn create_gitignore_test_file(data_to_write: String, path_to_write: &Path) {
        if !path_to_write.exists() {
            let mut output_file =
                File::create(path_to_write).expect("File wasn't successfully created");
            write!(output_file, "{}", data_to_write)
                .expect("Unable to write gitignore data to the file");
        }
    }

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

    #[test]
    fn test_read_gitignore() {
        // first create a test file : .test_gitignore (if doesn't exist already)
        let gitignore_file_path = Path::new(".test_gitignore");
        let data_to_expect = "target/*\n.git/*\n.test/*\nCargo.lock";
        let vec_data: Vec<String> = data_to_expect.split('\n').map(String::from).collect();
        create_gitignore_test_file(data_to_expect.to_string(), gitignore_file_path);

        if !gitignore_file_path.exists() {
            let mut output_file =
                File::create(gitignore_file_path).expect("File wasn't successfully created");
            write!(output_file, "{}", data_to_expect)
                .expect("Unable to write gitignore data to the file");
        }
        let data = read_gitignore(".test_gitignore".to_string());
        assert_eq!(data, vec_data);
    }

    #[test]
    fn test_check_if_should_be_ignored() {
        // first create a test file : .test_gitignore (if doesn't exist already)
        let gitignore_file_path = Path::new(".test_gitignore");
        let data_to_expect = "target/*\n.git/*\n.test/*\nCargo.lock";
        create_gitignore_test_file(data_to_expect.to_string(), gitignore_file_path);

        let expected_output_for_ignored = true;
        let expected_output_for_not_ignored = false;

        let data = read_gitignore(".test_gitignore".to_string());
        let output = check_if_should_be_ignored(Some(Path::new(".")), Path::new("./.git/"), &data);
        assert_eq!(output, expected_output_for_ignored);

        let output =
            check_if_should_be_ignored(Some(Path::new(".")), Path::new("./src/main.rs"), &data);
        assert_eq!(output, expected_output_for_not_ignored);

        // just making sure canonicalize function is working as expected
        let output = check_if_should_be_ignored(None, Path::new("./src/../src/main.rs"), &data);
        assert_eq!(output, expected_output_for_not_ignored);

        let output = check_if_should_be_ignored(
            Some(Path::new(&std::env::current_dir().unwrap())),
            Path::new("./src/../src/main.rs"),
            &data,
        );
        assert_eq!(output, expected_output_for_not_ignored);
    }
}
