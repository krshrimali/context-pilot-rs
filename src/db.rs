use crate::algo_loc;
use crate::git_command_algo::get_latest_commit;
use crate::git_command_algo::{get_commit_descriptions, get_commits_after, get_files_changed};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::{collections::HashMap, fs::File, path::Path};

use crate::algo_loc::perform_for_whole_file;
use crate::config::MAX_ITEMS_IN_EACH_DB_FILE;
use crate::contextgpt_structs::AuthorDetailsV2;
use crate::{config, contextgpt_structs::AuthorDetails};

// workspace_path: file_path: 1: AuthorDetails
type DBType = HashMap<String, HashMap<String, HashMap<u32, Vec<AuthorDetails>>>>;

// type DBTypeV2 = HashMap<String, HashMap<String, HashMap<u32, Vec<AuthorDetailsV2>>>>;
// {"line_number": [commit_hash_1, ...]}
type DBTypeV2 = HashMap<usize, Vec<String>>;

type MappingDBType = HashMap<String, Vec<u32>>;

#[allow(dead_code)]
#[derive(Default, Clone)]
pub struct DB {
    pub index: u32,                // The line of code that you are at, right now? TODO:
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
    pub indexing_file_name: String, // This is for storing the indexing metadata
}

#[allow(dead_code)]
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
        }
        init_data
    }

    // Initialise the DB if it doesn't exist already
    pub fn init_db(&mut self, workspace_path: &str, curr_file_path: Option<&str>, cleanup: bool) {
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
        // Check if self.folder_path exists, cleanup if cleanup is required.
        if cleanup && Path::new(&self.folder_path).exists() {
            // Remove the folder and all its contents
            std::fs::remove_dir_all(&self.folder_path)
                .unwrap_or_else(|_| panic!("Unable to remove the folder: {}", self.folder_path));
        }
        // Now initialise all relevant folders/files
        // When the child folders are also not present - we just want to iteratively create all folders
        std::fs::create_dir_all(&self.folder_path)
            .unwrap_or_else(|_| panic!("Unable to create folder for: {}", self.folder_path));

        // Search for the index
        let mut db_file_index: Option<Vec<u32>> = None;
        if curr_file_path.is_none() {
            db_file_index = self.find_index(workspace_path);
        } else {
            // convert curr_file_path to an absolute path:
            let curr_file_path = PathBuf::from(curr_file_path.unwrap());
            let curr_file_path = curr_file_path
                .canonicalize()
                .unwrap_or_else(|_| panic!("Unable to convert the path to absolute path"));
            db_file_index = self.find_index(curr_file_path.as_path().to_str().unwrap());
        }
        if db_file_index.is_none() {
            // No mapping yet - means no indexing hasn't happened yet.
            self.current_data_v2 = HashMap::new();
        } else {
            // let db_file_index = self.find_index(curr_file_path.unwrap_or(""));
            // Filename will be: <db_file_index>.json
            let valid_indices = db_file_index.unwrap_or(vec![self.index]);
            self.current_data_v2 = self.read_all(valid_indices.clone());
        }
    }

    fn read_mapping_file(&mut self) -> MappingDBType {
        // Read the mapping file from the folder_path
        self.mapping_file_name = "mapping.json".to_string();
        let mapping_path = format!("{}/{}", self.folder_path, self.mapping_file_name);
        self.mapping_file_path = String::from(mapping_path.clone());
        let mapping_path_obj = Path::new(&mapping_path);
        if !mapping_path_obj.exists() {
            eprintln!("Mapping file does not exist at: {}", mapping_path);
            self.mapping_data = HashMap::new();
            self.db_file_path = format!("{}/{}.json", self.folder_path, self.index);
            self.current_data = HashMap::new();
            return HashMap::new();
        }
        let mapping_data = match std::fs::read_to_string(&mapping_path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error reading file: {}: {}", mapping_path, e);
                return HashMap::new();
            }
        };
        serde_json::from_str(mapping_data.as_str()).unwrap_or_else(|_| {
            panic!(
                "Unable to deserialize the mapping file, path: {}",
                mapping_path
            )
        })
    }

    fn read_indexing_file(&mut self) -> HashMap<String, Vec<String>> {
        // Read the indexing file from the folder_path
        self.indexing_file_name = "indexing_metadata.json".to_string();
        let indexing_path = format!("{}/{}", self.folder_path, self.indexing_file_name);
        let indexing_path_obj = Path::new(&indexing_path);
        if !indexing_path_obj.exists() {
            // Does not exist: create a new file:
            eprintln!(
                "Indexing metadata file does not exist at: {}",
                indexing_path
            );
            // Create an empty file
            if let Err(e) = File::create(&indexing_path) {
                eprintln!("Error creating file: {}: {}", indexing_path, e);
            }
            return HashMap::new();
        }
        let indexing_data = match std::fs::read_to_string(&indexing_path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error reading file: {}: {}", indexing_path, e);
                return HashMap::new();
            }
        };
        serde_json::from_str(indexing_data.as_str()).unwrap_or_else(|_| {
            panic!(
                "Unable to deserialize the indexing metadata file, path: {}",
                indexing_path
            )
        })
    }

    pub fn find_index(&mut self, curr_file_path: &str) -> Option<Vec<u32>> {
        // In each folder -> we'll have a mapping file which contains which filename corresponds to which index (to be used in the DB file)
        // self.mapping_file_name = "mapping.json".to_string();
        // self.mapping_file_path = format!("{}/{}", self.folder_path, self.mapping_file_name);
        // let mapping_path_obj = Path::new(&self.mapping_file_path);
        // if !mapping_path_obj.exists() {
        //     self.mapping_data = HashMap::new();
        //     self.db_file_path = format!("{}/{}.json", self.folder_path, self.index);
        //     self.current_data = HashMap::new();
        //     return None;
        // }
        // let mapping_data = match std::fs::read_to_string(&self.mapping_file_path) {
        //     Ok(s) => s,
        //     Err(e) => {
        //         eprintln!("Error reading file: {}: {}", self.mapping_file_path, e);
        //         return None;
        //     }
        // };
        // let mut mapping_json: HashMap<String, Vec<u32>> =
        //     serde_json::from_str(mapping_data.as_str()).unwrap_or_else(|_| {
        //         panic!(
        //             "Unable to deserialize the mapping file, path: {}",
        //             self.mapping_file_path
        //         )
        //     });
        // let mapping_json_copy = mapping_json.clone();
        // let indices = mapping_json_copy.get(curr_file_path);
        // self.mapping_data = mapping_json.clone();
        self.mapping_data = self.read_mapping_file();
        if self.mapping_data.is_empty() {
            return None;
        }
        let indices = self.mapping_data.get(curr_file_path);
        let mut mapping_json = self.mapping_data.clone();
        if indices.is_none() {
            // The mapping file is there but we just don't have the corresponding entry for it
            // TODO: Store last available index for the DB
            // self.index = self.index + 1;
            self.index = self.get_available_index(&self.mapping_data);
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

    // pub fn last_indexed_commit(&self, mapping_json: &MappingDBType) -> Option<String> {
    //     // Generally, indexing would have already happened once, after which - we should just
    //     // attempt to index the new commits that have not been indexed yet. This function should
    //     // just return the last indexed commit. The logic using it should handle updating the mapping
    //     // post indexing any new commits from here.
    //     mapping_json.get("last_commit_indexed_commit").and_then(|v| v.first().copied())
    // }

    pub fn append_to_db(
        &mut self,
        configured_file_path: &String,
        _: usize,
        all_data: HashMap<u32, AuthorDetailsV2>,
    ) {
        self.curr_file_path = configured_file_path.clone();
        if all_data.is_empty() {
            return;
        }

        self.curr_items += all_data.len() as u32; // Just track number of items
        for line_number in all_data.keys() {
            let single_detail = all_data.get(line_number).unwrap().clone();
            self.current_data_v2
                .entry(single_detail.line_number)
                .or_default()
                .extend(single_detail.commit_hashes);
        }
    }

    pub fn _is_limit_crossed(&self) -> bool {
        self.curr_items >= MAX_ITEMS_IN_EACH_DB_FILE
    }

    pub fn store(&mut self) {
        if self.current_data_v2.is_empty() {
            eprintln!("No data to store.");
            return;
        }

        let db_file_path = format!("{}/{}.json", self.folder_path, self.index);
        self.mapping_data
            .entry(self.curr_file_path.clone())
            .or_default()
            .push(self.index);
        // Re-write the mapping file since data has changed:
        self.index += 1; // increment index for the next file.
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

        // Find the last commit hash for the current file
        let last_commit = get_latest_commit(&self.curr_file_path);

        self.prepare_indexing_metadata(&self.curr_file_path.clone(), &last_commit);

        self.current_data_v2.clear(); // clear everything after storing
        self.curr_items = 0; // reset
    }

    pub fn raw_exists_and_return(
        &mut self,
        start_line_number: &usize,
        end_line_number: &usize,
    ) -> (Vec<String>, Vec<u32>) {
        let mut all_commit_hashes = vec![];
        let mut uncovered_indices: Vec<u32> = vec![];
        // Find the "closest" maximum index to the given index.
        // Let's say if start_line_number is 5, and data is: ['3': [...], '7': [..., ...]]
        // Output you are looking for is of "7'"

        // Now get all the commit_hashes in the max_index entry.
        for i in *start_line_number..=*end_line_number {
            let mut max_index: Option<usize> = None;
            // Check if the exact index exists in our data
            if self.current_data_v2.contains_key(&i) {
                max_index = Some(i);
            } else {
                // Find index that is "closest" max to the given index.
                // This is the index that we will use to get the commit_hashes.
                let mut keys: Vec<_> = self.current_data_v2.keys().collect();
                keys.sort();
                for key in keys.iter() {
                    if **key > i {
                        max_index = Some(**key);
                        break;
                    }
                }
            }
            match max_index {
                Some(index) => {
                    if let Some(commit_hashes) = self.current_data_v2.get(&index) {
                        all_commit_hashes.extend(commit_hashes.clone());
                    }
                }
                None => {
                    // No data found for the given index
                    uncovered_indices.push(i as u32);
                }
            }
        }
        (all_commit_hashes, uncovered_indices)
    }

    pub fn exists_and_return(
        &mut self,
        start_line_number: &usize,
        end_line_number: &usize,
    ) -> (HashMap<String, usize>, Vec<u32>) {
        let mut uncovered_indices: Vec<u32> = vec![];

        // Find the "closest" maximum index to the given index.
        // Let's say if start_line_number is 5, and data is: ['3': [...], '7': [..., ...]]
        // Output you are looking for is of "7'"

        // Now get all the commit_hashes in the max_index entry.
        let mut counter_for_paths: HashMap<String, usize> = HashMap::new();
        for i in *start_line_number..=*end_line_number {
            let mut max_index: Option<usize> = None;
            // Check if the exact index exists in our data
            if self.current_data_v2.contains_key(&i) {
                max_index = Some(i);
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
                                *counter_for_paths.entry(rel_path.clone()).or_insert(0) += 1;
                            }
                        }
                    }
                }
                None => {
                    // No data found for the given index
                    uncovered_indices.push(i as u32);
                }
            }
        }
        (counter_for_paths, uncovered_indices)
    }

    fn update_last_indexed_commit(
        &mut self,
        file_path: &String,
        commit_hash: &str,
    ) -> Result<(), String> {
        // Read the existing indexing metadata
        let mut indexing_metadata = self.read_indexing_file();

        // Update or create the entry for this file path
        indexing_metadata
            .entry(file_path.clone())
            .or_insert_with(Vec::new)
            .push(commit_hash.to_string());

        // Write back to the indexing metadata file
        let indexing_path = format!("{}/{}", self.folder_path, self.indexing_file_name);

        // Serialize the updated metadata
        let indexing_string = serde_json::to_string_pretty(&indexing_metadata)
            .map_err(|e| format!("Failed to serialize indexing metadata: {}", e))?;

        // Write to file
        std::fs::write(&indexing_path, indexing_string)
            .map_err(|e| format!("Failed to write indexing metadata: {}", e))?;

        Ok(())
    }

    fn prepare_indexing_metadata(&mut self, file_path: &String, last_commit_hash: &Option<String>) {
        // If we have a valid commit hash, update the indexing metadata
        if let Some(commit) = last_commit_hash {
            if let Err(e) = self.update_last_indexed_commit(file_path, commit) {
                eprintln!("Failed to update indexing metadata: {}", e);
            }
        }
    }

    pub async fn query(&mut self, file_path: String, start_number: usize, end_number: usize) {
        let mut end_line_number = end_number;
        if end_number == 0 {
            // Means, cover the whole file.
            // end_number should be the last line number of the file.
            end_line_number = std::fs::read_to_string(&file_path)
                .unwrap_or_else(|_| panic!("Unable to read the file: {}", file_path))
                .lines()
                .count();
        }
        if self.current_data_v2.is_empty() {
            // No data to query - means no indexing has happened yet.
            // Let's treat this as a binary and perform operation.
            let output =
                algo_loc::perform_for_whole_file(file_path.clone(), false, None, None).await;
            let mut commit_hashes = vec![];
            for line_number in output.keys() {
                let struct_detail = output.get(line_number).unwrap();
                // Check if struct_details' line number comes b/w start_number and end_number:
                if struct_detail.line_number >= start_number
                    && struct_detail.line_number <= end_line_number
                {
                    commit_hashes.extend(struct_detail.commit_hashes.clone());
                }
            }
            // Now iterate through the commit hashes:
            let mut counter_for_paths: HashMap<String, usize> = HashMap::new();
            for commit_hash in commit_hashes.iter() {
                // Compute contextual file paths using the commit hash.
                // We use git show for this.
                let relevant_file_paths = get_files_changed(commit_hash);
                // Add each file path and increment count if it already existed.
                for rel_path in relevant_file_paths.iter() {
                    *counter_for_paths.entry(rel_path.clone()).or_insert(0) += 1;
                }
            }
            println!("Commit hashes found: {:?}", commit_hashes);
            // Write the last commit hash to the index metadata.
            let last_commit_hash = commit_hashes.last().unwrap().to_string();
            self.prepare_indexing_metadata(&file_path, &Some(last_commit_hash));
            for (path, count) in counter_for_paths.iter() {
                println!("{} - {} occurrences", path, count);
            }
        } else {
            // Generally - check first if the last indexed commit is the same as the current one.
            // If it is, then we can just return the data from the DB.
            let recent_commit = get_latest_commit(&file_path).unwrap();
            // Read the mapping file first from self.mapping_file_path
            let indexing_metadata = self.read_indexing_file();
            let last_indexing_data =
                indexing_metadata
                    .get(&file_path.clone())
                    .unwrap_or_else(|| {
                        panic!("No indexing metadata found for the file: {}", file_path);
                    });
            let last_indexed_commit = last_indexing_data.last().cloned();
            if last_indexed_commit.is_some() {
                if last_indexed_commit.clone().unwrap().eq(&recent_commit) {
                    // No need to index again, just return the data from the DB.
                    // eprintln!("No new commits to index, returning existing data.");
                } else {
                    // Index the new commits and update the DB.
                    // First get the new commits that have not been indexed yet.
                    let commits_to_index = get_commits_after(last_indexed_commit.unwrap());
                    // Index these commits first.
                    perform_for_whole_file(file_path.clone(), false, Some(commits_to_index), None)
                        .await;
                }
            }

            let (relevant_paths_with_counter, _uncovered_indices) =
                self.exists_and_return(&start_number, &end_line_number);

            for (path, count) in relevant_paths_with_counter.iter() {
                println!("{} - {} occurrences", path, count);
            }
        }
    }

    pub async fn query_descriptions(
        &mut self,
        file_path: String,
        start_number: usize,
        end_number: usize,
    ) {
        let mut end_line_number = end_number;
        if end_number == 0 {
            // Means, cover the whole file.
            // end_number should be the last line number of the file.
            end_line_number = std::fs::read_to_string(&file_path)
                .unwrap_or_else(|_| panic!("Unable to read the file: {}", file_path))
                .lines()
                .count();
        }
        if self.current_data_v2.is_empty() {
            // No data to query - means no indexing has happened yet.
            // Let's treat this as a binary and perform the operation ourselves:
            let output =
                algo_loc::perform_for_whole_file(file_path.clone(), false, None, None).await;
            let mut commit_hashes = vec![];
            for line_number in output.keys() {
                // Check if struct_details' line number comes b/w start_number and end_number:
                let struct_detail = output.get(line_number).unwrap();
                if struct_detail.line_number >= start_number
                    && struct_detail.line_number <= end_line_number
                {
                    commit_hashes.extend(struct_detail.commit_hashes.clone());
                }
            }
            // Get commit descriptions for these hashes
            let out = get_commit_descriptions(commit_hashes);
            println!("{:?}", out);
        } else {
            // Generally - check first if the last indexed commit is the same as the current one.
            // If it is, then we can just return the data from the DB.
            let recent_commit = get_latest_commit(&file_path).unwrap();
            // Read the mapping file first from self.mapping_file_path
            let indexing_metadata = self.read_indexing_file();
            let last_indexing_data =
                indexing_metadata
                    .get(&file_path.clone())
                    .unwrap_or_else(|| {
                        panic!("No indexing metadata found for the file: {}", file_path);
                    });
            let last_indexed_commit = last_indexing_data.last().cloned();
            if last_indexed_commit.is_some() {
                if last_indexed_commit.clone().unwrap().eq(&recent_commit) {
                    // No need to index again, just return the data from the DB.
                    // eprintln!("No new commits to index, returning existing data.");
                } else {
                    // Index the new commits and update the DB.
                    // First get the new commits that have not been indexed yet.
                    let commits_to_index = get_commits_after(last_indexed_commit.unwrap());
                    // Index these commits first.
                    perform_for_whole_file(file_path.clone(), false, Some(commits_to_index), None)
                        .await;
                }
            }

            let (commit_hashes, _uncovered_indices) =
                self.raw_exists_and_return(&start_number, &end_line_number);

            let out = get_commit_descriptions(commit_hashes);
            println!("{:?}", out);
        }
    }
}

// mod test {
//     use super::*;
//
//     #[test]
//     fn test_loading_mapping_file() {
//         let mapping_path = "/home/krshrimali/.context_pilot_db/mapping.json";
//         let mapping_data = std::fs::read_to_string(mapping_path).unwrap_or_else(|_| {
//             panic!(
//                 "Unable to read the mapping file into string, file path: {}",
//                 mapping_path
//             )
//         });
//         let mapping_path_obj = Path::new(mapping_path);
//         serde_json::from_str(mapping_data.as_str()).unwrap_or_else(|_| {
//             panic!(
//                 "Unable to deserialize the mapping file, path: {}",
//                 mapping_path
//             )
//         });
//
//         assert!(mapping_path_obj.exists());
//     }
// }
