use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::{collections::HashMap, fs::File, path::Path};
use crate::git_command_algo::get_files_changed;
use uuid;
use std::process::{Command, Stdio};
use std::sync::Arc;
use crossbeam::thread;

// use simple_home_dir::home_dir;

use crate::config::MAX_ITEMS_IN_EACH_DB_FILE;
use crate::contextgpt_structs::AuthorDetailsV2;
use crate::{config, contextgpt_structs::AuthorDetails};

// workspace_path: file_path: 1: AuthorDetails
type DBType = HashMap<String, HashMap<String, HashMap<u32, Vec<AuthorDetails>>>>;

// type DBTypeV2 = HashMap<String, HashMap<String, HashMap<u32, Vec<AuthorDetailsV2>>>>;
// {"line_number": [commit_hash_1, ...]}
type DBTypeV2 = HashMap<usize, Vec<String>>;

type MappingDBType = HashMap<String, Vec<u32>>;
type MappingDBTypeV2 = HashMap<String, String>;

// index; folder_path; currLines;
#[derive(Default, Clone)]
pub struct DB {
    pub index: u32, // The line of code that you are at, right now? TODO:
    pub folder_path: String, // Current folder path that this DB is processing, or the binary is running
    pub curr_items: u32,     // TODO:
    pub mapping_file_name: String, // This is for storing which file is in which folder/file? <-- TODO:
    pub current_data: DBType, // The data that we have from the loaded DB into our inhouse member
    pub current_data_v2: DBTypeV2,
    pub db_file_path: String,
    pub mapping_file_path: String,
    pub mapping_data: MappingDBType,
    // pub mapping_data_v2: MappingDBTypeV2,
    pub curr_file_path: String,
    pub workspace_path: String,
}

impl DB {
    pub fn read(&mut self) -> DBTypeV2 {
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

    pub fn read_all(&mut self, valid_indices: Vec<u32>) -> DBTypeV2 {
        let mut init_data: DBTypeV2 = HashMap::new();
        // let folder_path = self.folder_path.clone();
        // let workspace_path = self.workspace_path.clone();
        // let curr_file_path = self.curr_file_path.clone();

        // TODO: Clean this up, this is bad! valid_indices later on should just be a single file.
        for &valid_index in valid_indices.iter() {
            self.index = valid_index;

            self.db_file_path = format!("{}/{}.json", self.folder_path, self.index);
            let db_obj = Path::new(&self.db_file_path);
            if !db_obj.exists() {
                File::create(db_obj).expect("Couldn't find the DB file");
                continue;
                // return HashMap::new();
            }
            let current_data_v2 = self.read();
            init_data.extend(current_data_v2.clone());
            // let valid_values = current_data_v2
            //     .get(&workspace_path.clone())
            //     .and_then(|ws| ws.get(&curr_file_path.clone()))
            //     .cloned();
            //
            // let workspace_data = current_data_v2
            //     .entry(workspace_path.clone())
            //     .or_insert_with(HashMap::new);
            //
            // let file_data = workspace_data
            //     .entry(curr_file_path.clone())
            //     .or_insert_with(HashMap::new);
            // //
            // // Extend the file data with valid_values if present.
            // if let Some(valid_values) = valid_values {
            //     file_data.extend(valid_values.clone());
            // }
            // init_data.insert(workspace_path.clone(), workspace_data.clone());
        }
        init_data
    }

    // Initialise the DB if it doesn't exist already
    pub fn init_db(&mut self, workspace_path: &str, curr_file_path: Option<&str>) {
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
        self.current_data_v2 = self.read_all(valid_indices.clone());
    }

    fn find_index(&mut self, curr_file_path: &str) -> Option<Vec<u32>> {
        // In case the input path is empty.
        // if curr_file_path.is_empty() {
        //     return None;
        // }

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
        _: usize,
        all_data: Vec<AuthorDetailsV2>,
    ) {
        self.curr_file_path = configured_file_path.clone();
        if all_data.is_empty() {
            return;
        }

        self.curr_items += all_data.len() as u32; // Just track number of items
        for single_detail in all_data {
            self.current_data_v2
                .entry(single_detail.line_number)
                .or_insert_with(Vec::new)
                .extend(single_detail.commit_hashes);
        }
    }

    pub fn _is_limit_crossed(&self) -> bool {
        self.curr_items >= MAX_ITEMS_IN_EACH_DB_FILE
    }

    pub fn store(&mut self) {
        let CHUNK_SIZE = 30;
        if self.current_data_v2.is_empty() {
            eprintln!("No data to store.");
            return;
        }

        let db_file_path = format!("{}/{}.json", self.folder_path, self.index);
        self.index += 1; // increment index for the next file.
        self.mapping_data
            .entry(self.curr_file_path.clone())
            .or_insert_with(Vec::new)
            .push(self.index);
        let output_string = serde_json::to_string(&self.current_data_v2);
        if let Err(e) = std::fs::write(&db_file_path, output_string.unwrap()) {
            eprintln!("❌ Failed writing DB file {}: {}", db_file_path, e);
        } else {
            println!("✅ Successfully stored shard: {}", db_file_path);
        }

        // Update mapping file
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

        self.current_data_v2.clear(); // clear everything after storing
        self.curr_items = 0; // reset
    }

    pub fn exists_and_return(
        &mut self,
        search_field_first: &String,
        start_line_number: &usize,
        end_line_number: &usize,
    ) -> (HashMap<String, usize>, Vec<u32>) {
        // let mut already_computed_data: Vec<AuthorDetails> = vec![];
        let mut uncovered_indices: Vec<u32> = vec![];

        // // Normalize path to absolute
        // let abs_search_path = PathBuf::from(search_field_first)
        //     .canonicalize()
        //     .unwrap_or_else(|_| {
        //         panic!("Failed to canonicalize search path: {}", search_field_first)
        //     })
        //     .to_str()
        //     .unwrap()
        //     .to_string();

        // Find the "closest" maximum index to the given index.
        // Let's say if start_line_number is 5, and data is: ['3': [...], '7': [..., ...]]
        // Output you are looking for is of "7'"

        // Now get all the commit_hashes in the max_index entry.
        let mut counter_for_paths: HashMap<String, usize> = HashMap::new();
        for i in *start_line_number..=*end_line_number {
            let mut max_index: Option<usize> = None;
            for key in self.current_data_v2.keys() {
                if *key > i {
                    max_index = Some(*key);
                } else {
                    break;
                }
            }
            match max_index {
                Some(index) => {
                    if let Some(commit_hashes) = self.current_data_v2.get(&index) {
                        // If the index is present, we can get the commit_hashes
                        // and add them to the counter_for_paths.
                        for commit_hash in commit_hashes {
                            // Compute contextual file paths using the commit hash.
                            // We use git show for this.
                            let relevant_file_paths = get_files_changed(commit_hash);
                            // Add each file path and increment count if it already existed.
                            for rel_path in relevant_file_paths.iter() {
                                *counter_for_paths
                                    .entry(rel_path.clone())
                                    .or_insert(0) += 1;
                            }
                        }
                    }
                }
                None => {
                    // No data found for the given index
                    // uncovered_indices.extend((*start_line_number..=*end_line_number).map(|idx| idx as u32));
                    uncovered_indices.push(i as u32);
                }
            }
        }
        (counter_for_paths, uncovered_indices)

        // if let Some(workspace_data) = self.current_data.get_mut(&self.workspace_path) {
        //     if let Some(file_line_data) = workspace_data.get_mut(&abs_search_path) {
        //         for each_line_idx in *start_line_number..=*end_line_number {
        //             let each_line_idx = each_line_idx as u32;
        //             if let Some(eligible_data) = file_line_data.get_mut(&each_line_idx) {
        //                 already_computed_data.append(eligible_data);
        //             } else {
        //                 uncovered_indices.push(each_line_idx);
        //             }
        //         }
        //     } else {
        //         // File not present under current workspace
        //         uncovered_indices
        //             .extend((*start_line_number..=*end_line_number).map(|idx| idx as u32));
        //     }
        // } else {
        //     // Workspace not found
        //     uncovered_indices.extend((*start_line_number..=*end_line_number).map(|idx| idx as u32));
        // }

        // if already_computed_data.is_empty() {
        //     (None, uncovered_indices)
        // } else {
        //     (Some(already_computed_data), uncovered_indices)
        // }
    }

    pub fn query(&mut self, file_path: String, start_number: usize, end_number: usize) {
        let mut end_line_number = end_number;
        if end_number == 0 {
            // Means, cover the whole file.
            // end_number should be the last line number of the file.
            end_line_number = std::fs::read_to_string(&file_path)
                .unwrap_or_else(|_| panic!("Unable to read the file: {}", file_path))
                .lines()
                .count();
        }
        let (relevant_paths_with_counter, _uncovered_indices) =
            self.exists_and_return(&file_path, &start_number, &end_line_number);

        for (path, count) in relevant_paths_with_counter.iter() {
            println!("{} - {} occurrences", path, count);
        }

        // match maybe_results {
        //     Some(results) => {
        //         let mut relevance_counter: HashMap<String, usize> = HashMap::new();
        //
        //         for author_detail in results {
        //             for path in &author_detail.contextual_file_paths {
        //                 *relevance_counter.entry(path.clone()).or_insert(0) += 1;
        //             }
        //         }
        //
        //         let mut contextual_paths: Vec<(&String, &usize)> =
        //             relevance_counter.iter().collect();
        //         contextual_paths.sort_by(|a, b| b.1.cmp(a.1)); // Sort by descending relevance
        //
        //         for (path, count) in contextual_paths {
        //             println!("{} - {} occurrences", path, count);
        //         }
        //     }
        //     None => {
        //         println!("⚠️ No matching data found for: {}", file_path);
        //     }
        // }
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
