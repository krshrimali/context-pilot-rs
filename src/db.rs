use std::fs::OpenOptions;
use std::io::Write;
use std::{collections::HashMap, fs::File, path::Path};

// use simple_home_dir::home_dir;

use crate::config::MAX_ITEMS_IN_EACH_DB_FILE;
use crate::{config, contextgpt_structs::AuthorDetails};

type DBType = HashMap<String, HashMap<u32, Vec<AuthorDetails>>>;
type MappingDBType = HashMap<String, Vec<u32>>;

// index; folder_path; currLines;
#[derive(Default)]
pub struct DB {
    pub index: u32,                // The line of code that you are at, right now? TODO:
    pub folder_path: String, // Current folder path that this DB is processing, or the binary is running
    pub curr_items: u32,     // TODO:
    pub mapping_file_name: String, // This is for storing which file is in which folder/file? <-- TODO:
    pub current_data: DBType, // The data that we have from the loaded DB into our inhouse member
    pub db_file_path: String,
    pub mapping_file_path: String,
    pub mapping_data: MappingDBType,
    pub curr_file_path: String,
}

impl DB {
    pub fn read(&mut self) -> DBType {
        // let db_file_path = format!("{}/{}", self.folder_path, self.index);
        if Path::new(self.db_file_path.as_str()).exists() {
            let data_buffers =
                std::fs::read_to_string(&self.db_file_path).expect("Unable to read the file");
            let data: DBType =
                serde_json::from_str(data_buffers.as_str()).expect("Unable to deserialize");
            data
        } else {
            // TODO: Add a log that the DB doesn't exist
            // TODO: Enable logging into a logging file and add two modes: debug and info
            HashMap::new()
        }
    }

    pub fn read_all(&mut self, valid_indices: Vec<u32>) -> DBType {
        let mut init_data: DBType = HashMap::new();
        for valid_index in valid_indices.iter() {
            self.index = *valid_index;
            self.db_file_path = format!("{}/{}.json", self.folder_path, self.index);
            let db_obj = Path::new(&self.db_file_path);
            if !db_obj.exists() {
                File::create(db_obj).expect("Couldn't find the DB file");
                return HashMap::new();
            }
            let current_data = self.read();
            let values = current_data.get(&self.curr_file_path);
            if let Some(valid_values) = values {
                let mut default: HashMap<u32, Vec<AuthorDetails>> = HashMap::new();
                let insert_data_here = init_data
                    .get_mut(&self.curr_file_path)
                    .unwrap_or(&mut default);
                insert_data_here.extend(valid_values.clone());
                let copy_data = insert_data_here.clone();
                init_data.insert(self.curr_file_path.clone(), copy_data);
            }
        }
        init_data
    }

    // Initialise the DB if it doesn't exist already
    pub fn init_db(&mut self, curr_file_path: &str) {
        // let folder_path = Path::new(simple_home_dir::home_dir().unwrap().to_str().unwrap())
        //     .join(config::DB_FOLDER);
        // let db_folder = config::DB_FOLDER.to_owned() + &self.folder_path;
        let db_folder = format!("{}/{}", config::DB_FOLDER, self.folder_path);
        self.curr_file_path = String::from(curr_file_path);
        // let mut main_folder_path = String::new();

        if let Some(home) = simple_home_dir::home_dir() {
            let folder_path = home.join(db_folder);

            if let Some(path_str) = folder_path.to_str() {
                self.folder_path = path_str.to_string();
            } else {
                eprintln!("Something went wrong while trying to get the string for the path");
                return;
            }
        } else {
            eprintln!("Failed to determine the home directory");
            return;
        }
        self.index = 0;
        self.curr_items = 0;

        // Now initialise all relevant folders/files
        // When the child folders are also not present - we just want to iteratively create all folders
        std::fs::create_dir_all(&self.folder_path)
            .unwrap_or_else(|_| panic!("Unable to create folder for: {}", self.folder_path));

        // Search for the index
        let db_file_index = self.find_index(curr_file_path);
        // Filename will be: <db_file_index>.json
        let valid_indices = db_file_index.unwrap_or(vec![self.index]);
        self.current_data = self.read_all(valid_indices.clone());
    }

    fn find_index(&mut self, curr_file_path: &str) -> Option<Vec<u32>> {
        // In each folder -> we'll have a mapping file which contains which filename corresponds to which index (to be used in the DB file)
        self.mapping_file_name = "mapping.json".to_string();
        self.mapping_file_path = format!("{}/{}", self.folder_path, self.mapping_file_name);
        let mapping_path_obj = Path::new(&self.mapping_file_path);
        if !mapping_path_obj.exists() {
            // mapping file doesn't exist yet... we'll create one with the index as 0 for the given curr_file_path
            let mut data: MappingDBType = HashMap::new();
            data.insert(curr_file_path.to_string(), vec![self.index]);
            self.mapping_data = data.clone();
            self.mapping_data
                .insert(String::from("last_used_index"), [self.index].to_vec());
            let init_mapping_string =
                serde_json::to_string_pretty(&self.mapping_data).expect("Unable to create data");
            let mut mapping_path_file: File =
                File::create(&self.mapping_file_path).expect("Couldn't create this new file...");
            write!(mapping_path_file, "{}", init_mapping_string)
                .expect("Couldn't write a very simple data object into a new mapping file...wow!");
            self.db_file_path = format!("{}/{}.json", self.folder_path, self.index);
            self.current_data = HashMap::new();
            return None;
        }
        let mapping_data = std::fs::read_to_string(&self.mapping_file_path).unwrap_or_else(|_| {
            panic!(
                "Unable to read the mapping file into string, file path: {}",
                self.mapping_file_path
            )
        });
        let mut mapping_json: HashMap<String, Vec<u32>> =
            serde_json::from_str(mapping_data.as_str()).unwrap_or_else(|_| {
                panic!(
                    "Unable to deserialize the mapping file, path: {}",
                    self.mapping_file_path
                )
            });
        let mapping_json_copy = mapping_json.clone();
        let indices = mapping_json_copy.get(curr_file_path);
        self.mapping_data = mapping_json.clone();
        if indices.is_none() {
            // The mapping file is there but we just don't have the corresponding entry for it
            // TODO: Store last available index for the DB
            // self.index = self.index + 1;
            self.index = self.get_available_index(&mapping_json);

            self.db_file_path = format!("{}/{}.json", self.folder_path, self.index);
            mapping_json.insert(curr_file_path.to_string(), vec![self.index]);
            self.mapping_data = mapping_json.clone();
            let init_mapping_string =
                serde_json::to_string_pretty(&mapping_json).expect("Unable to create data");
            let mut file = OpenOptions::new()
                .write(true)
                .append(false) // TODO: Would love to append here instead
                .open(&self.mapping_file_path)
                .unwrap();
            writeln!(file, "{}", init_mapping_string)
                .expect("Couldn't write to the mapping file, wow!");
            return None;
        }
        indices.cloned()
    }

    pub fn get_available_index(&self, mapping_json: &HashMap<String, Vec<u32>>) -> u32 {
        let default_vec = &[0_u32].to_vec();
        let last_used_index: &Vec<u32> = mapping_json.get("last_used_index").unwrap_or(default_vec);
        last_used_index[0]
    }

    pub fn append(
        &mut self,
        configured_file_path: &String,
        start_line_idx: usize,
        end_line_idx: usize,
        all_data: Vec<AuthorDetails>,
    ) {
        for line_idx in start_line_idx..end_line_idx + 1 {
            let line_idx = line_idx as u32;
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

    pub fn _is_limit_crossed(&self) -> bool {
        self.curr_items >= MAX_ITEMS_IN_EACH_DB_FILE
    }

    pub fn store(&mut self) {
        // We should check if the limit has crossed and then modify self.db_file_path
        let mut we_crossed_limit: bool = false;
        self.curr_items += 1;
        if self._is_limit_crossed() {
            // We'll have to make sure that the new file is created
            self.curr_items = 0;
            self.index += 1;
            self.db_file_path = format!("{}/{}.json", self.folder_path, self.index);

            // Updating mapping content and file as well
            self.mapping_data
                .insert(String::from("last_used_index"), [self.index].to_vec());

            let list_indices_before = self.mapping_data.get_mut(&self.curr_file_path);
            list_indices_before
                .unwrap_or(&mut vec![])
                .append(&mut vec![self.index]);

            let mapping_string = serde_json::to_string_pretty(&self.mapping_data)
                .expect("Unable to deserialize data");
            // Update the mapping file accordingly
            let mut file = File::create(&self.mapping_file_path)
                .expect("Couldn't create the mapping file for some reason.");
            writeln!(file, "{}", mapping_string).expect("Couldn't write to the mapping file, wow!");
            we_crossed_limit = true;
        }

        let output_string =
            serde_json::to_string_pretty(&self.current_data).expect("Unable to deserialize data");
        if we_crossed_limit {
            self.current_data.clear();
        }
        if Path::new(&self.db_file_path).exists() {
            let mut file_obj = File::create(self.db_file_path.as_str())
                .unwrap_or_else(|_| panic!("Couldn't open the given file: {}", self.db_file_path));
        } else {
            let mut file_obj = File::create(self.db_file_path.as_str())
                .unwrap_or_else(|_| panic!("Couldn't open the given file: {}", self.db_file_path));
        }
    }

    pub fn exists_and_return(
        &mut self,
        search_field_first: &String,
        start_line_number: &usize,
        end_line_number: &usize,
    ) -> (Option<Vec<&AuthorDetails>>, Vec<u32>) {
        let mut already_computed_data: Vec<&AuthorDetails> = vec![];
        let mut uncovered_indices: Vec<u32> = vec![];
        if self.current_data.contains_key(search_field_first) {
            let output = self.current_data.get_mut(search_field_first);
            if let Some(all_line_data) = output {
                for each_line_idx in *start_line_number..*end_line_number + 1 {
                    let each_line_idx = each_line_idx as u32;
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
                uncovered_indices.push(idx as u32);
            }
            (None, uncovered_indices)
        }
    }
}
