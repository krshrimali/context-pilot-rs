use std::{collections::HashMap, fs::File, io::Write, path::Path};

use crate::{config, contextgpt_structs::AuthorDetails};

// Type of the DB stored, to be used in this file
type DBType = HashMap<String, HashMap<usize, Vec<AuthorDetails>>>;

#[derive(Default)]
pub struct DB {
    pub db_file_name: String,
    pub current_data: DBType,
    pub db_file_path: String,
}

impl DB {
    pub fn read(&mut self) -> DBType {
        let data_buffer = std::fs::read_to_string(self.db_file_path.clone()).unwrap();
        let v: DBType = serde_json::from_str(data_buffer.as_str())
            .expect("Unable to deserialize the file, something went wrong");
        v
    }

    pub fn init_db(&mut self) {
        let folder_path = Path::new(simple_home_dir::home_dir().unwrap().to_str().unwrap())
            .join(config::DB_FOLDER);
        self.db_file_path = folder_path
            .join(&self.db_file_name)
            .to_str()
            .unwrap()
            .to_string();
        // Create folder
        std::fs::create_dir_all(folder_path)
            .expect("unable to create folder, something went wrong");
        let db_path_obj: &Path = Path::new(&self.db_file_path);
        if !db_path_obj.exists() {
            File::create(db_path_obj).expect("Couldn't create the file for some reason");
            self.current_data = HashMap::new();
            return;
        }
        self.current_data = self.read();
    }

    pub fn append(
        &mut self,
        configured_file_path: &String,
        start_line_idx: usize,
        end_line_idx: usize,
        all_data: Vec<AuthorDetails>,
    ) {
        for line_idx in start_line_idx..end_line_idx + 1 {
            let mut existing_data = vec![];
            if self.current_data.contains_key(configured_file_path) {
                let file_data = self.current_data.get_mut(configured_file_path).unwrap();
                match file_data.contains_key(&line_idx) {
                    false => {
                        file_data.insert(line_idx, all_data.clone());
                    }
                    true => {
                        file_data
                            .get_mut(&line_idx)
                            .unwrap()
                            .append(&mut all_data.clone());
                    }
                }
            } else {
                existing_data.extend(all_data.clone());
                let mut map = HashMap::new();
                map.insert(line_idx, existing_data);
                self.current_data
                    .insert(configured_file_path.to_string(), map);
            }
        }
    }

    pub fn store(&mut self) {
        let mut file_obj =
            File::create(self.db_file_path.as_str()).expect("Couldn't open the given file");
        let output_string =
            serde_json::to_string_pretty(&self.current_data).expect("Unable to write data");
        write!(file_obj, "{}", output_string).expect("Couldn't write, uhmmm");
    }

    pub fn exists_and_return(
        &mut self,
        search_field_first: &String,
        start_line_number: &usize,
        end_line_number: &usize,
    ) -> (Option<Vec<&AuthorDetails>>, Vec<usize>) {
        let mut already_computed_data: Vec<&AuthorDetails> = vec![];
        let mut uncovered_indices: Vec<usize> = vec![];
        if self.current_data.contains_key(search_field_first) {
            let output = self.current_data.get_mut(search_field_first);
            if let Some(all_line_data) = output {
                for each_line_idx in *start_line_number..*end_line_number + 1 {
                    if let Some(eligible_data) = all_line_data.get(&each_line_idx) {
                        already_computed_data.extend(eligible_data);
                    } else {
                        uncovered_indices.push(each_line_idx);
                    }
                }
            }
            if already_computed_data.is_empty() {
                (None, uncovered_indices)
            } else {
                (Some(already_computed_data), uncovered_indices)
            }
        } else {
            for idx in *start_line_number..*end_line_number + 1 {
                uncovered_indices.push(idx);
            }
            (None, uncovered_indices)
        }
    }
}
