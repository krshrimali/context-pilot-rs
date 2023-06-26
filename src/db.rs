use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    path::Path,
};

use crate::{config, contextgpt_structs::AuthorDetails};

#[derive(Default)]
pub struct DB {
    pub db_file_name: String,
    pub current_data: HashMap<String, Vec<AuthorDetails>>,
    pub db_file_path: String,
}

impl DB {
    pub fn read(&mut self) -> HashMap<String, Vec<AuthorDetails>> {
        let data_buffer = std::fs::read_to_string(self.db_file_path.clone()).unwrap();
        let v: HashMap<String, Vec<AuthorDetails>> = serde_json::from_str(data_buffer.as_str())
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

    pub fn append(&mut self, configured_file_path: &String, data: AuthorDetails) {
        let mut existing_data = vec![];
        if self.current_data.contains_key(configured_file_path) {
            existing_data = self
                .current_data
                .get_mut(configured_file_path)
                .unwrap()
                .to_vec();
            existing_data.append(&mut vec![data]);
        } else {
            existing_data.append(&mut vec![data]);
        }
        self.current_data
            .insert(configured_file_path.to_string(), existing_data);
    }

    pub fn store(&mut self) {
        let mut file_obj =
            File::create(self.db_file_path.as_str()).expect("Couldn't open the given file");
        let output_string =
            serde_json::to_string_pretty(&self.current_data).expect("Unable to write data");
        // file_obj
        //     .write_all(output_string.as_bytes())
        //     .expect("Unable to write bytes to the file");
        write!(file_obj, "{}", output_string).expect("Couldn't write, uhmmm");
    }

    pub fn exists(&self, search_field: &String) -> Option<&Vec<AuthorDetails>> {
        if self.current_data.contains_key(search_field) {
            self.current_data.get(search_field)
        } else {
            None
        }
    }
}
