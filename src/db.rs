use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::{collections::HashMap, fs::File, path::Path};

// use simple_home_dir::home_dir;

use crate::config::MAX_ITEMS_IN_EACH_DB_FILE;
use crate::{config, contextgpt_structs::AuthorDetails};

// workspace_path: file_path: 1: AuthorDetails
type DBType = HashMap<String, HashMap<String, HashMap<u32, Vec<AuthorDetails>>>>;
type MappingDBType = HashMap<String, Vec<u32>>;

// index; folder_path; currLines;
#[derive(Default, Clone)]
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
    pub workspace_path: String,
}

impl DB {
    pub fn read(&mut self) -> DBType {
        // let db_file_path = format!("{}/{}", self.folder_path, self.index);
        if Path::new(self.db_file_path.as_str()).exists() {
            let data_buffers = match std::fs::read_to_string(&self.db_file_path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error reading file: {}", e);
                    return HashMap::new();
                }
            };
            match serde_json::from_str(&data_buffers) {
                Ok(data) => data,
                Err(err) => {
                    eprintln!("Failed to parse JSON: {}", err);
                    HashMap::new()
                }
            }
        } else {
            eprintln!(
                "The DB file doesn't exist for the given path: {}",
                self.db_file_path
            );
            // TODO: Enable logging into a logging file and add two modes: debug and info
            HashMap::new()
        }
    }

    pub fn read_all(&mut self, valid_indices: Vec<u32>) -> DBType {
        let mut init_data: DBType = HashMap::new();
        // let folder_path = self.folder_path.clone();
        let workspace_path = self.workspace_path.clone();
        let curr_file_path = self.curr_file_path.clone();

        for &valid_index in valid_indices.iter() {
            self.index = valid_index;

            self.db_file_path = format!("{}/{}.json", self.folder_path, self.index);
            let db_obj = Path::new(&self.db_file_path);
            if !db_obj.exists() {
                File::create(db_obj).expect("Couldn't find the DB file");
                continue;
                // return HashMap::new();
            }
            let mut current_data = self.read();
            let valid_values = current_data
                .get(&workspace_path.clone())
                .and_then(|ws| ws.get(&curr_file_path.clone()))
                .cloned();

            let workspace_data = current_data
                .entry(workspace_path.clone())
                .or_insert_with(HashMap::new);

            let file_data = workspace_data
                .entry(curr_file_path.clone())
                .or_insert_with(HashMap::new);
            //
            // Extend the file data with valid_values if present.
            if let Some(valid_values) = valid_values {
                file_data.extend(valid_values.clone());
            }
            init_data.insert(workspace_path.clone(), workspace_data.clone());
        }
        init_data
    }

    // Initialise the DB if it doesn't exist already
    pub fn init_db(&mut self, workspace_path: &str, curr_file_path: Option<&str>) {
        // let folder_path = Path::new(simple_home_dir::home_dir().unwrap().to_str().unwrap())
        //     .join(config::DB_FOLDER);
        // let db_folder = config::DB_FOLDER.to_owned() + &self.folder_path;
        let db_folder = format!("{}/{}", config::DB_FOLDER, self.folder_path);
        self.workspace_path = String::from(workspace_path);
        self.curr_file_path = String::from(curr_file_path.unwrap_or(""));
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
        let mut db_file_index: Option<Vec<u32>> = None;
        if curr_file_path.is_none() {
            db_file_index = self.find_index(workspace_path);
        } else {
            db_file_index = self.find_index(curr_file_path.unwrap());
        }
        // let db_file_index = self.find_index(curr_file_path.unwrap_or(""));
        // Filename will be: <db_file_index>.json
        let valid_indices = db_file_index.unwrap_or(vec![self.index]);
        self.current_data = self.read_all(valid_indices.clone());
    }

    fn find_index(&mut self, curr_file_path: &str) -> Option<Vec<u32>> {
        // In case the input path is empty.
        // if curr_file_path.is_empty() {
        //     return None;
        // }

        // In each folder -> we'll have a mapping file which contains which filename corresponds to which index (to be used in the DB file)
        self.mapping_file_name = "mapping.json".to_string();
        self.mapping_file_path = format!("{}/{}", self.folder_path, self.mapping_file_name);
        // println!("Doing this\n");
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
        let mapping_data = match std::fs::read_to_string(&self.mapping_file_path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error reading file: {}: {}", self.mapping_file_path, e);
                return None;
            }
        };
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
        mapping_json
            .get("last_used_index")
            .and_then(|v| v.first().copied())
            .unwrap_or_else(|| {
                let max_index = mapping_json
                    .values()
                    .flat_map(|v| v.iter().copied())
                    .max()
                    .unwrap_or(0);
                max_index + 1
            })
    }

    pub fn append_to_db(
        &mut self,
        configured_file_path: &String,
        start_line_idx: usize,
        all_data: Vec<AuthorDetails>,
    ) {
        println!("Appending for: {:?}", configured_file_path);
        if all_data.is_empty() {
            return;
        }

        let end_line_idx = all_data[0].end_line_number;

        let workspace_entry = self
            .current_data
            .entry(self.workspace_path.clone())
            .or_insert_with(HashMap::new);

        let file_entry = workspace_entry
            .entry(configured_file_path.clone())
            .or_insert_with(HashMap::new);

        for line_idx in start_line_idx..=end_line_idx {
            let line_idx = line_idx as u32;

            file_entry
                .entry(line_idx)
                .or_insert_with(Vec::new)
                .extend(all_data.clone());
        }
    }

    // pub fn append_to_db(
    //     &mut self,
    //     configured_file_path: &String,
    //     start_line_idx: usize,
    //     all_data: Vec<AuthorDetails>,
    // ) {
    //     println!("Appending for: {:?}", configured_file_path.clone());
    //     if all_data.is_empty() {
    //         return;
    //     }
    //
    //     let end_line_idx = all_data[0].end_line_number;
    //     for line_idx in start_line_idx..end_line_idx + 1 {
    //         let line_idx = line_idx as u32;
    //         let mut existing_data = vec![];
    //         // println!("self.current-data: {:?}", self.current_data);
    //         if self.current_data.contains_key(&self.workspace_path) {
    //             let workspace_data = self.current_data.get_mut(&self.workspace_path).unwrap();
    //             if workspace_data.contains_key(configured_file_path) {
    //                 let file_data = workspace_data.get_mut(configured_file_path).unwrap();
    //                 if file_data.contains_key(&line_idx) {
    //                     existing_data = file_data.get(&line_idx).unwrap().clone();
    //                     println!("AHHH OKAYYY");
    //                     // TODO: Add this to self.current_data once testing is successful.
    //                 }
    //                 match file_data.contains_key(&line_idx) {
    //                     false => {
    //                         file_data.insert(line_idx, all_data.clone());
    //                     }
    //                     true => {
    //                         file_data
    //                             .get_mut(&line_idx)
    //                             .unwrap()
    //                             .append(&mut all_data.clone());
    //                     } // config file; db_+size: 306
    //                       // config_file_2: size; 108
    //                 }
    //             } else {
    //                 let mut another_map: HashMap<u32, Vec<AuthorDetails>> = HashMap::new();
    //                 let mut to_insert_map: HashMap<String, HashMap<u32, Vec<AuthorDetails>>> =
    //                     HashMap::new();
    //                 another_map.insert(line_idx, all_data.clone());
    //                 // workspace_data.insert(configured_file_path.clone(), another_map);
    //                 to_insert_map.insert(configured_file_path.clone(), another_map);
    //                 self.current_data
    //                     .insert(self.workspace_path.clone(), to_insert_map);
    //             }
    //         } else {
    //             existing_data.extend(all_data.clone());
    //             let mut to_insert_map: HashMap<String, HashMap<u32, Vec<AuthorDetails>>> =
    //                 HashMap::new();
    //             let mut another_map: HashMap<u32, Vec<AuthorDetails>> = HashMap::new();
    //             another_map.insert(line_idx, existing_data);
    //             // println!("Configurwed file path: {}", configured_file_path);
    //             to_insert_map.insert(configured_file_path.clone(), another_map);
    //             self.current_data
    //                 .insert(self.workspace_path.clone(), to_insert_map);
    //         }
    //     }
    // }

    pub fn _is_limit_crossed(&self) -> bool {
        self.curr_items >= MAX_ITEMS_IN_EACH_DB_FILE
    }
    pub fn store(&mut self) {
        if self.current_data.is_empty() {
            println!("No data to store.");
            return;
        }

        self.curr_items += 1;
        let mut we_crossed_limit = false;

        if self._is_limit_crossed() {
            self.curr_items = 0;
            self.index += 1;
            self.db_file_path = format!("{}/{}.json", self.folder_path, self.index);
            we_crossed_limit = true;

            // Update mapping index
            self.mapping_data
                .insert("last_used_index".to_string(), vec![self.index]);

            // Safely update indices list for current file
            self.mapping_data
                .entry(self.curr_file_path.clone())
                .or_insert_with(Vec::new)
                .push(self.index);

            // Write updated mapping file
            if let Ok(mut file) = File::create(&self.mapping_file_path) {
                let mapping_string = serde_json::to_string_pretty(&self.mapping_data)
                    .expect("Failed to serialize mapping");
                if let Err(e) = write!(file, "{}", mapping_string) {
                    eprintln!("❌ Failed writing mapping: {}", e);
                }
            } else {
                eprintln!(
                    "❌ Failed to create mapping file: {}",
                    self.mapping_file_path
                );
            }
        }

        // Debug: What is being stored?
        for (workspace_path, files_map) in &self.current_data {
            println!("🔑 Workspace: {}", workspace_path);
            for file in files_map.keys() {
                println!("  └── File: {}", file);
            }
        }

        // Write DB file
        let output_string = serde_json::to_string_pretty(&self.current_data)
            .expect("Failed to serialize DB content");

        if let Err(e) = std::fs::write(&self.db_file_path, output_string) {
            eprintln!("❌ Failed writing DB to {}: {}", self.db_file_path, e);
        } else {
            println!("✅ Stored DB to {}", self.db_file_path);
        }

        if we_crossed_limit {
            self.current_data.clear();
        }
    }

    pub fn exists_and_return(
        &mut self,
        search_field_first: &String,
        start_line_number: &usize,
        end_line_number: &usize,
    ) -> (Option<Vec<AuthorDetails>>, Vec<u32>) {
        let mut already_computed_data: Vec<AuthorDetails> = vec![];
        let mut uncovered_indices: Vec<u32> = vec![];
        // println!("Searching for the given field: {}", search_field_first);
        if self.current_data.contains_key(&self.workspace_path.clone()) {
            let check = self.current_data.get_mut(&self.workspace_path.clone());
            println!("check: {:?}", check.unwrap().clone().keys());
            let output = self
                .current_data
                .get_mut(&self.workspace_path.clone())
                .unwrap()
                .get_mut(search_field_first);
            println!("output length: {:?}", output);

            // Do some sanitization and ensure every item in output contains the key as search_field_first
            if let Some(all_line_data) = output {
                for each_line_idx in *start_line_number..*end_line_number + 1 {
                    let each_line_idx = each_line_idx as u32;
                    if let Some(eligible_data) = all_line_data.get_mut(&each_line_idx) {
                        already_computed_data.append(eligible_data);
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

    pub fn query(&mut self, file_path: String, start_number: usize, end_number: usize) {
        println!("Querying the DB for the given file path: {}", file_path);
        println!("End number: {}", end_number);
        println!("Start number: {}", start_number);
        println!("File path: {}", file_path);
        let output = self.exists_and_return(&file_path, &start_number, &end_number);
        println!("output: {:?}", output);
        println!("Final leng: {:?}", output.0.unwrap().len());
        return;
    }
}

mod test {
    use super::*;

    #[test]
    fn test_loading_mapping_file() {
        let mapping_path = "/home/krshrimali/.context_pilot_db/mapping.json";
        let mapping_data = std::fs::read_to_string(mapping_path).unwrap_or_else(|_| {
            panic!(
                "Unable to read the mapping file into string, file path: {}",
                mapping_path
            )
        });
        let mapping_path_obj = Path::new(mapping_path);
        serde_json::from_str(mapping_data.as_str()).unwrap_or_else(|_| {
            panic!(
                "Unable to deserialize the mapping file, path: {}",
                mapping_path
            )
        });

        assert!(mapping_path_obj.exists());
    }
}
