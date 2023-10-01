// use crate::{config, contextgpt_structs::AuthorDetails};

use std::{collections::HashMap, fs::File, path::Path};

// use simple_home_dir::home_dir;

use crate::{config, contextgpt_structs::AuthorDetails};

type DBType = HashMap<String, HashMap<usize, Vec<AuthorDetails>>>;

// index; folder_path; currLines;
#[derive(Default)]
pub struct DB {
    pub index: u32,
    pub folder_path: String,
    pub curr_lines: u32,
    pub mapping_file_name: String,
    pub current_data: DBType,
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
            return HashMap::new();
        }
    }

    // Initialise the DB if it doesn't exist already
    pub fn init_db(&mut self, curr_file_path: &str) {
        // let folder_path = Path::new(simple_home_dir::home_dir().unwrap().to_str().unwrap())
        //     .join(config::DB_FOLDER);
        let db_folder = config::DB_FOLDER;

        if let Some(home) = simple_home_dir::home_dir() {
            let folder_path = home.join(db_folder);

            if let Some(path_str) = folder_path.to_str() {
                self.folder_path = path_str.to_string();
            }
        } else {
            eprintln!("Failed to determine the home directory");
            return;
        }
        self.index = 0;
        self.curr_lines = 0;
        self.mapping_file_name = String::from("mapping.json");

        // Now initialise all relevant folders/files
        std::fs::create_dir_all(self.folder_path).expect(&format!(
            "Unable to create folder for: {}",
            self.folder_path
        ));

        // Search for the index
        let db_file_index = self.find_index(curr_file_path);
        if let Some(valid_index) = db_file_index {
            self.index = *valid_index;
            let db_file_path = format!("{}/{}", self.folder_path, self.index);
            let db_obj = Path::new(&db_file_path);
            if !db_obj.exists() {
                File::create(db_obj).expect("Couldn't create the file for some reason");
                self.current_data = HashMap::new();
                return;
            }
            self.current_data = self.read();
        } else {
            self.current_data = HashMap::new();
            return;
        }
    }

    fn find_index(&mut self, curr_file_path: &str) -> Option<&u32> {
        let mapping_file_path = format!("{}/{}", self.folder_path, self.mapping_file_name);
        let mapping_data = std::fs::read_to_string(mapping_file_path).expect(&format!(
            "Unable to read the mapping file into string, file path: {}",
            mapping_file_path
        ));
        let mapping_json: HashMap<String, u32> = serde_json::from_str(mapping_data.as_str())
            .expect(&format!(
                "Unable to deserialize the mapping file, path: {}",
                mapping_file_path
            ));
        return mapping_json.get(curr_file_path);
    }
}
