use std::{collections::HashMap, fs::File, io::Write, path::Path};

use crate::{config, contextgpt_structs::AuthorDetails};

#[derive(Default)]
pub struct DB {
    pub db_file_name: String,
    pub current_data: HashMap<String, HashMap<String, Vec<AuthorDetails>>>,
    pub db_file_path: String,
}

impl DB {
    pub fn read(&mut self) -> HashMap<String, HashMap<String, Vec<AuthorDetails>>> {
        let data_buffer = std::fs::read_to_string(self.db_file_path.clone()).unwrap();
        let v: HashMap<String, HashMap<String, Vec<AuthorDetails>>> =
            serde_json::from_str(data_buffer.as_str())
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
        start_line_number: usize,
        end_line_number: usize,
        data: AuthorDetails,
    ) {
        let mut existing_data = vec![];
        let line_str: String = format!("{start_line_number}_{end_line_number}");
        if self.current_data.contains_key(configured_file_path) {
            existing_data = self
                .current_data
                .get_mut(configured_file_path)
                .unwrap()
                .get_mut(&line_str)
                .unwrap()
                .to_vec();
            existing_data.append(&mut vec![data]);
        } else {
            existing_data.append(&mut vec![data]);
            self.current_data
                .insert(configured_file_path.to_string(), HashMap::new());
        }
        self.current_data
            .get_mut(configured_file_path)
            .unwrap()
            .insert(line_str, existing_data);
    }

    pub fn store(&mut self) {
        let mut file_obj =
            File::create(self.db_file_path.as_str()).expect("Couldn't open the given file");
        let output_string =
            serde_json::to_string_pretty(&self.current_data).expect("Unable to write data");
        write!(file_obj, "{}", output_string).expect("Couldn't write, uhmmm");
    }

    pub fn exists(
        &self,
        search_field_first: &String,
        search_field_second: &String,
    ) -> (Option<Vec<AuthorDetails>>, String) {
        if self.current_data.contains_key(search_field_first) {
            let line_numbers: Vec<&str> = search_field_second.split('_').collect();
            let start_line_number: usize = line_numbers.first().unwrap().parse().unwrap();
            let end_line_number: usize = line_numbers.last().unwrap().parse().unwrap();
            let file_searched = self.current_data.get(search_field_first);
            match file_searched {
                Some(existing_lines) => {
                    let keys = existing_lines.keys();
                    if keys.len() == 0 {
                        return (None, search_field_second.to_string());
                    }
                    let mut output_vec = None;
                    let mut output_string = "".to_string();
                    for each_key_combination in keys {
                        let line_numbers: Vec<&str> = each_key_combination.split('_').collect();
                        let received_start_line_number: usize =
                            line_numbers.first().unwrap().parse().unwrap();
                        let received_end_line_number: usize =
                            line_numbers.last().unwrap().parse().unwrap();
                        if start_line_number == received_start_line_number
                            && end_line_number == received_end_line_number
                        {
                            output_vec = existing_lines.get(each_key_combination).cloned();
                            output_string = "".to_string();
                        } else if start_line_number > received_start_line_number
                            && end_line_number < received_end_line_number
                        {
                            // in between
                            let full_data = existing_lines.get(each_key_combination).unwrap();
                            let mut final_data: Vec<AuthorDetails> = Vec::new();
                            for line_data in full_data {
                                if line_data.line_number >= start_line_number
                                    && line_data.line_number <= end_line_number
                                {
                                    final_data.push(line_data.clone());
                                }
                            }
                            output_vec = Some(final_data);
                            output_string = "".to_string();
                        } else if start_line_number > received_start_line_number
                        // && end_line_number > received_start_line_number
                        {
                            let full_data = existing_lines.get(each_key_combination).unwrap();
                            let mut final_data: Vec<AuthorDetails> = Vec::new();
                            for line_data in full_data {
                                if line_data.line_number > start_line_number
                                    && line_data.line_number <= received_end_line_number
                                {
                                    final_data.push(line_data.clone());
                                }
                            }
                            output_vec = Some(final_data);
                            let final_start_line_number = received_end_line_number + 1;
                            output_string = format!("{final_start_line_number}_{end_line_number}");
                        } else if start_line_number < received_start_line_number
                            && end_line_number > received_start_line_number
                        {
                            let full_data = existing_lines.get(each_key_combination).unwrap();
                            let mut final_data: Vec<AuthorDetails> = Vec::new();
                            for line_data in full_data {
                                if line_data.line_number > received_start_line_number
                                    && line_data.line_number <= end_line_number
                                {
                                    final_data.push(line_data.clone());
                                }
                            }
                            output_vec = Some(final_data);
                            if end_line_number > received_end_line_number {
                                let final_received_end_line_number = received_end_line_number + 1;
                                output_string = format!("{start_line_number}_{received_start_line_number}_{final_received_end_line_number}_{end_line_number}");
                            } else {
                                output_string =
                                    format!("{start_line_number}_{received_start_line_number}");
                            }
                        } else {
                            output_vec = None;
                            output_string = search_field_second.to_string();
                        }
                    }
                    (output_vec, output_string)
                }
                _ => (None, search_field_second.to_string()),
            }
        } else {
            (None, search_field_second.to_string())
        }
    }
}
