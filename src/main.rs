mod algo_loc;
mod authordetails_impl;
mod config;
mod config_impl;
mod contextgpt_structs;
mod db;
mod diff_v2;
mod git_command_algo;

use crate::{algo_loc::perform_for_whole_file, db::DB};
use async_recursion::async_recursion;
use contextgpt_structs::{AuthorDetailsV2, Cli, RequestTypeOptions};
use std::collections::HashMap;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use structopt::StructOpt;
use tokio::sync::Mutex;

use quicli::prelude::{
    log::{log, Level},
    CliResult,
};
use tokio::task;

#[derive(Default, Debug, Eq, PartialEq, Clone, Copy)]
pub enum State {
    Starting, // Indexing alr
    Running,  // Indexing finished
    #[default]
    Dead, // Server is not running
    Failed,   // On any failure, but the server is still running
}

#[derive(Clone)]
pub struct DBHandler {
    metadata: DBMetadata,
}

impl DBHandler {
    fn new(metadata: DBMetadata) -> DBHandler {
        Self { metadata }
    }

    fn _is_eligible(&mut self, _: &PathBuf) -> bool {
        true
    }

    // TODO: this is something I'm repeating in the server as well
    // I should ideally move this to a common place
    // And also store this somewhere so that I don't re-compute
    fn _valid_file_count(&mut self, folder_path: &str) -> i64 {
        let mut total_count: i64 = 0;
        for entry in std::fs::read_dir(folder_path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                if self._is_eligible(&path) {
                    total_count += 1;
                } else {
                    log!(Level::Warn, "File is not valid: {}", path.display());
                }
            }
        }
        total_count
    }

    pub fn init(&mut self, folder_path: &str) {
        // iterate through folder_path and calculate total number of valid files
        // and set the metadata
        let total_valid_file_count = self._valid_file_count(folder_path);

        self.metadata = DBMetadata {
            // Initial state should be stopped or..?
            state: State::Dead,
            workspace_path: folder_path.to_string(),
            curr_progress: 0,
            total_count: total_valid_file_count,
        };
    }

    pub fn retry(&mut self) {
        todo!();
    }

    pub fn get_current_metadata(&mut self) -> DBMetadata {
        self.metadata.clone()
    }

    pub fn start(&mut self, _: &DBMetadata) {
        // this should ideally start the DB server
        // DB Server and the other server should be kept separate
        // this should not be async though - as we'll really want this to finish before it finishes
        // println!(
        //     "Passing workspace path to init_db: {}",
        //     metadata.workspace_path
        // );
    }
}

#[derive(Clone)]
pub struct Server {
    state: State,
    curr_db: Option<Arc<Mutex<DB>>>,
    state_db_handler: DBHandler,
}

#[derive(Default, Debug, Clone)]
pub struct DBMetadata {
    state: State,
    workspace_path: String,
    curr_progress: i64, // file index you're at OR percentage done
    total_count: i64,   // how many files are indexing
}

impl DBMetadata {}

impl Server {
    // Constructor
    fn new(state: State, db_handler: DBHandler) -> Server {
        Self {
            state,
            curr_db: None,
            state_db_handler: db_handler,
        }
    }

    fn init_server(&mut self, curr_db: Arc<Mutex<DB>>) {
        self.curr_db = Some(curr_db);
    }

    fn _is_valid_file(file: &Path) -> bool {
        if file.exists() && file.is_file() {
            // not optimising one liners here for debugging later on
            log!(Level::Debug, "File exists: {}", file.display());
            return true;
        }
        false
    }

    async fn _index_file(file_path_inp: PathBuf) -> Vec<AuthorDetailsV2> {
        // Don't make it write to the DB, write it atomically later.
        // For now, just store the output somewhere in the DB.
        let file_path = std::fs::canonicalize(file_path_inp).expect("Failed");
        let file_path_str = file_path.to_str().unwrap();
        // println!("Calling file\n");

        // Read the config file and pass defaults
        // let config_obj: config_impl::Config = config_impl::read_config(config::CONFIG_FILE_NAME);

        // curr_db.init_db(file_path);
        let output_author_details = perform_for_whole_file(file_path_str.to_string(), true).await;

        // for each_output in output_author_details.iter() {
        //     if each_output.origin_file_path != file_path_str {
        //         panic!("Something went wrong while indexing file and this is not expected.");
        //     }
        // }
        // TODO: (@krshrimali) Add this back.
        // Now extract output string from the output_author_details.
        // extract_string_from_output(output_author_details, /*is_author_mode=*/ false)
        output_author_details
    }

    #[async_recursion]
    async fn _iterate_through_workspace(
        &mut self,
        workspace_path: PathBuf,
    ) -> Vec<AuthorDetailsV2> {
        let mut files_set: task::JoinSet<Vec<AuthorDetailsV2>> = task::JoinSet::new();
        let path = Path::new(&workspace_path);
        let mut final_authordetails: Vec<AuthorDetailsV2> = Vec::new();

        if path.is_dir() {
            for entry in path
                .read_dir()
                .unwrap_or_else(|_| panic!("failed reading directory {}", path.display()))
            {
                let curr_db = self.curr_db.clone();
                let entry_path = entry.unwrap().path();
                if entry_path.is_dir() {
                    files_set.spawn({
                        let entry_path_clone = entry_path.clone();
                        let state_db_handler_clone = self.state_db_handler.clone();
                        let curr_db_clone = curr_db.clone();
                        async move {
                            let mut server = Server {
                                state: State::Running,
                                curr_db: curr_db_clone,
                                state_db_handler: state_db_handler_clone,
                            };
                            let result = server
                                ._iterate_through_workspace(entry_path_clone.clone())
                                .await;
                            result
                        }
                    });
                } else {
                    if Server::_is_valid_file(&entry_path) {
                        log!(Level::Info, "File is valid: {}", entry_path.display());
                        files_set.spawn({
                            async move {
                                let output = Server::_index_file(entry_path.clone()).await;
                                output
                            }
                        });
                    }
                }
            }

            while let Some(res) = files_set.join_next().await {
                let output_authordetails = res.unwrap();

                if output_authordetails.is_empty() {
                    continue;
                }

                // 🛠 Group by file path and update DB
                use std::collections::HashMap;
                let mut grouped_by_file: HashMap<String, Vec<AuthorDetailsV2>> = HashMap::new();

                for detail in output_authordetails {
                    grouped_by_file
                        .entry(detail.origin_file_path.clone())
                        .or_default()
                        .push(detail);
                }

                for (origin_file_path, details_vec) in grouped_by_file {
                    if details_vec.is_empty() {
                        continue;
                    }
                    let db = self.curr_db.clone().unwrap();
                    let mut db_locked = db.lock().await;
                    let start_line_number = 0;
                    db_locked.append_to_db(
                        &origin_file_path,
                        start_line_number,
                        details_vec.clone(),
                    );
                    db_locked.store();
                    final_authordetails.extend(details_vec);
                }
            }
        } else {
            if Server::_is_valid_file(path) {
                log!(
                    Level::Warn,
                    "File is valid but not in a sub-directory: {}",
                    path.display()
                );
                let output = Server::_index_file(path.to_path_buf()).await;
                return output;
            } else {
                log!(Level::Warn, "File is not valid: {}", path.display());
            }
        }

        final_authordetails
    }

    pub async fn start_file(&mut self, _: &mut DBMetadata, _: Option<String>) {
        return;
    }

    pub async fn start_indexing(&mut self, metadata: &mut DBMetadata) {
        // start the server for the given workspace
        // TODO: see if you just want to pass the workspace path and avoiding passing the whole metadata here
        let workspace_path = &metadata.workspace_path;

        // the server will start going through all the "valid" files in the workspace and will index them
        // defn of valid: files that satisfy .gitignore check (if exists and not disabled in the config)
        // and files that are not in the ignore list if provided in the config

        let workspace_path_buf = PathBuf::from(workspace_path);
        // First check if indexing is already done - if yes, just cleanup and restart.
        // Check if mapping.json exists.
        let db = DB {
            folder_path: workspace_path.clone(),
            ..Default::default()
        };
        let curr_db: Arc<Mutex<DB>> = Arc::new(db.into());
        curr_db
            .lock()
            .await
            .init_db(workspace_path.as_str(), None, /*cleanup=*/ true);
        let mut server = Server::new(State::Dead, DBHandler::new(metadata.clone()));
        server.init_server(curr_db);
        let _ = server
            ._iterate_through_workspace(workspace_path_buf.clone())
            .await;
    }

    pub async fn handle_server(
        &mut self,
        workspace_path: &str,
        file_path: Option<String>,
        start_number: Option<usize>,
        end_number: Option<usize>,
        request_type: Option<RequestTypeOptions>,
    ) {
        // this will initialise any required states
        self.state_db_handler.init(workspace_path);
        let mut metadata = self.state_db_handler.get_current_metadata();

        // If this is a call to query and not to index ->
        if request_type.is_some() && request_type.clone().unwrap() == RequestTypeOptions::Query {
            let db = DB {
                folder_path: workspace_path.to_string().clone(),
                ..Default::default()
            };
            let curr_db: Arc<Mutex<DB>> = Arc::new(db.into());
            curr_db.lock().await.init_db(
                workspace_path,
                file_path.clone().as_deref(),
                /*cleanp=*/ false,
            );
            // let mut server = Server::new(State::Dead, DBHandler::new(metadata.clone()));
            self.init_server(curr_db);
            // Then you query
            // assert!(file_path.is_some());
            assert!(start_number.is_some());
            assert!(end_number.is_some());
            assert!(self.curr_db.is_some());
            self.curr_db.clone().unwrap().lock().await.query(
                file_path.clone().unwrap(),
                start_number.unwrap(),
                end_number.unwrap(),
            ).await;
        } else if request_type.is_some () && request_type.unwrap() == RequestTypeOptions::Descriptions {
            let db = DB {
                folder_path: workspace_path.to_string().clone(),
                ..Default::default()
            };
            let curr_db: Arc<Mutex<DB>> = Arc::new(db.into());
            curr_db.lock().await.init_db(
                workspace_path,
                file_path.clone().as_deref(),
                /*cleanp=*/ false,
            );
            // let mut server = Server::new(State::Dead, DBHandler::new(metadata.clone()));
            self.init_server(curr_db);
            assert!(file_path.is_some());
            assert!(start_number.is_some());
            assert!(end_number.is_some());
            assert!(self.curr_db.is_some());
            self.curr_db.clone().unwrap().lock().await.query_descriptions(
                file_path.clone().unwrap(),
                start_number.unwrap(),
                end_number.unwrap(),
            ).await;
        }

        let mut tasks = vec![];

        if metadata.workspace_path == workspace_path {
            if metadata.state == State::Running {
                // the server is already running
                // do nothing - let the indexing continue
                // println!("[CONTINUING] Progress: {}", metadata.curr_progress);
            } else if metadata.state == State::Starting {
                // the server is not in running state -> for state of it's attempting to start -> let it finish and then see if it was successful
                // TODO: @krshrimali
                self.state_db_handler.start(&metadata);
                tasks.push(self.start_file(&mut metadata, file_path));
            } else if metadata.state == State::Failed {
                // in case of failure though, ideally it would have been alr handled by other process -> but in any case, starting from here as well to just see how it works out
                // I'm in the favor of not restarting in case of failure from another process though
                // TODO: Have a limit here, can't retry for infinite count.
                self.state_db_handler.retry();
            } else if metadata.state == State::Dead {
                // should be blocking rn
                self.state_db_handler.start(&metadata);
                match file_path {
                    None => {
                        if workspace_path.is_empty() {
                            panic!("no workspace path passed")
                        }
                        self.start_indexing(&mut metadata).await;
                    }
                    _ => {
                        self.start_file(&mut metadata, file_path).await;
                    }
                }
            }
        }
    }
}

// NOTE: @krshrimali - use this for testing. Output should always be 616 lines.
// #[tokio::main]
// async fn main() {
//     let commit_hashes = [
//         "2e76e1e", "4face15", "6d9e881", "42baffc", "8b6e985", "eed8bd9", "49dad60", "e14b51d",
//         "c79810d", "72e52c1", "e6c4521", "0ab4f85", "c50c6d7", "1d6961a", "67c7369", "e41324a",
//         "7393de7",
//     ];
//     // let commit_hashes = [
//     //     "2e76e1e", "4face15"
//     // ];
//     let mut map: HashMap<u32, Vec<diff_v2::LineDetail>> = HashMap::new();
//     let file_name = "src/diff_v2.rs";
//     for commit_hash in commit_hashes.iter() {
//         diff_v2::extract_commit_hashes(commit_hash, &mut map, file_name);
//         // for key in map.keys() {
//         //     if map.get(&key).unwrap().len() == 1 {
//         //         if map.get(&key).unwrap()[0].commit_hashes.len() > 1  {
//         //             println!("Key: {:?}", key);
//         //             println!("Value: {:?}", map.get(&key).unwrap()[0].commit_hashes);
//         //
//         //         }
//         //     }
//         // }
//     }
// }

#[tokio::main]
async fn main() -> CliResult {
    let args = Cli::from_args();

    env_logger::init();
    let mut server = Server {
        state: State::Dead,
        curr_db: None,
        state_db_handler: DBHandler {
            metadata: DBMetadata::default(),
        },
    };

    // TODO: Add support for config file.
    // let config_obj: config_impl::Config = config_impl::read_config(config::CONFIG_FILE_NAME);
    // let mut file_path: Option<PathBuf> = None;
    // if args.file.is_some() {
    //     file_path = PathBuf::from_str(args.file.unwrap().as_str())
    //         .unwrap()
    //         .into();
    // }

    match args.request_type {
        RequestTypeOptions::File => {
            server
                .handle_server(args.folder_path.as_str(), args.file, None, None, None)
                .await;
        }
        RequestTypeOptions::Author => {
            server
                .handle_server(args.folder_path.as_str(), args.file, None, None, None)
                .await;
        }
        RequestTypeOptions::Index => {
            server
                .handle_server(args.folder_path.as_str(), None, None, None, None)
                .await;
        }
        RequestTypeOptions::Query => {
            server
                .handle_server(
                    args.folder_path.as_str(),
                    args.file,
                    args.start_number,
                    args.end_number,
                    Some(RequestTypeOptions::Query),
                )
                .await;
        }
        RequestTypeOptions::Descriptions => {
            server
                .handle_server(args.folder_path.as_str(), args.file, args.start_number, args.end_number, Some(RequestTypeOptions::Descriptions))
                .await;
        }
    };
    Ok(())
}
