// use crate::{config, contextgpt_structs::AuthorDetails};

use std::io::Write;
use std::{collections::HashMap, fs::File, path::Path};

// use simple_home_dir::home_dir;

use crate::{config, contextgpt_structs::AuthorDetails};

type DBType = HashMap<String, HashMap<usize, Vec<AuthorDetails>>>;
type MappingDBType = HashMap<String, usize>;

// index; folder_path; currLines;
#[derive(Default)]
pub struct DB {
    pub index: u32,                // The line of code that you are at, right now? TODO:
    pub folder_path: String, // Current folder path that this DB is processing, or the binary is running
    pub curr_lines: u32,     // TODO:
    pub mapping_file_name: String, // This is for storing which file is in which folder/file? <-- TODO:
    pub current_data: DBType, // The data that we have from the loaded DB into our inhouse member
}

impl DB {
    pub fn read(&mut self) -> DBType {
        let db_file_path = format!("{}/{}", self.folder_path, self.index);
        if Path::new(db_file_path.as_str()).exists() {
            let data_buffers =
                std::fs::read_to_string(db_file_path).expect("Unable to read the file");
            let data: DBType =
                serde_json::from_str(data_buffers.as_str()).expect("Unable to deserialize");
            return data;
        } else {
            // TODO: Add a log that the DB doesn't exist
            // TODO: Enable logging into a logging file and add two modes: debug and info
            return HashMap::new();
        }
    }

    // Initialise the DB if it doesn't exist already
    pub fn init_db(&mut self, curr_file_path: &str) {
        // let folder_path = Path::new(simple_home_dir::home_dir().unwrap().to_str().unwrap())
        //     .join(config::DB_FOLDER);
        // let db_folder = config::DB_FOLDER.to_owned() + &self.folder_path;
        let db_folder = format!("{}/{}", config::DB_FOLDER, self.folder_path);
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
        self.curr_lines = 0;
        // self.mapping_file_name = String::from("mapping.json");

        // Now initialise all relevant folders/files
        // When the child folders are also not present - we just want to iteratively create all folders
        std::fs::create_dir_all(&self.folder_path).expect(&format!(
            "Unable to create folder for: {}",
            self.folder_path
        ));

        // Search for the index
        let db_file_index = self.find_index(curr_file_path);
        // Filename will be: <db_file_index>.json
        if let Some(valid_index) = db_file_index {
            self.index = valid_index;
            let db_file_path = format!("{}/{}.json", self.folder_path, self.index);
            let db_obj = Path::new(&db_file_path);
            if !db_obj.exists() {
                File::create(db_obj).expect("Couldn't find the DB file");
                self.current_data = HashMap::new();
                return;
            }
            self.current_data = self.read();
        } else {
            self.current_data = HashMap::new();
            // return;
        }
    }

    fn find_index(&mut self, curr_file_path: &str) -> Option<u32> {
        // In each folder -> we'll have a mapping file which contains which filename corresponds to which index (to be used in the DB file)
        self.mapping_file_name = "mapping.json".to_string();
        let mapping_file_path = format!("{}/{}", self.folder_path, self.mapping_file_name);
        let mapping_path_obj = Path::new(&mapping_file_path);
        if !mapping_path_obj.exists() {
            // mapping file doesn't exist yet... we'll create one with the index as 0 for the given curr_file_path
            let mut data: MappingDBType = HashMap::new();
            data.insert(curr_file_path.to_string(), 0);
            // let mut data = format!("{{}: 0}", curr_file_path);
            let init_mapping_string =
                serde_json::to_string_pretty(&data).expect("Unable to create data");
            let mut mapping_path_file =
                File::create(mapping_file_path).expect("Couldn't create this new file...");
            write!(mapping_path_file, "{}", init_mapping_string)
                .expect("Couldn't write a very simple data object into a new file...wow!");
            return None;
        }
        let mapping_data = std::fs::read_to_string(&mapping_file_path).expect(&format!(
            "Unable to read the mapping file into string, file path: {}",
            mapping_file_path
        ));
        let mapping_json: HashMap<String, u32> = serde_json::from_str(mapping_data.as_str())
            .expect(&format!(
                "Unable to deserialize the mapping file, path: {}",
                mapping_file_path
            ));
        mapping_json.get(curr_file_path).copied()
    }
}
