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
use git_command_algo::print_all_valid_files;
use std::collections::HashMap;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use structopt::StructOpt;
use tokio::sync::Mutex;

use ignore::gitignore::{Gitignore, GitignoreBuilder};
use quicli::prelude::{
    CliResult,
    log::{Level, log},
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
            folders_to_index: vec![],
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
    folders_to_index: Vec<String>,
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

    async fn _index_file(file_path_inp: PathBuf) -> HashMap<u32, AuthorDetailsV2> {
        // Don't make it write to the DB, write it atomically later.
        // For now, just store the output somewhere in the DB.
        let file_path = std::fs::canonicalize(file_path_inp).expect("Failed");
        let file_path_str = file_path.to_str().unwrap();
        perform_for_whole_file(file_path_str.to_string(), true).await
    }

    #[async_recursion]
    async fn _iterate_through_workspace(
        &mut self,
        workspace_path: PathBuf,
        gitignore_builder_obj: Option<Gitignore>,
    ) -> HashMap<u32, AuthorDetailsV2> {
        let mut files_set: task::JoinSet<HashMap<u32, AuthorDetailsV2>> = task::JoinSet::new();
        let path = Path::new(&workspace_path);
        let mut final_authordetails: HashMap<u32, AuthorDetailsV2> = HashMap::new();
        if path.is_dir() {
            for entry in path
                .read_dir()
                .unwrap_or_else(|_| panic!("failed reading directory {}", path.display()))
            {
                let curr_db = self.curr_db.clone();
                let entry_path_path = entry.unwrap().path();
                let entry_path_str = entry_path_path.to_str().unwrap();
                let to_strip = format!("{}{}", self.state_db_handler.metadata.workspace_path, "/");
                let entry_path_stripped = entry_path_str
                    .strip_prefix(to_strip.as_str())
                    .unwrap_or(entry_path_str)
                    .to_string();
                // Check if entry_path matches gitignore pattern - ignore if yes.
                if let Some(gitignore_obj) = gitignore_builder_obj.clone() {
                    // Strip workspace path + '/' from the entry_path if it's not relative:
                    if gitignore_obj
                        .matched(&entry_path_stripped, true)
                        .is_ignore()
                    {
                        continue;
                    }
                }
                if entry_path_path.is_dir() {
                    files_set.spawn({
                        let entry_path_clone = entry_path_path.clone();
                        let state_db_handler_clone = self.state_db_handler.clone();
                        let curr_db_clone = curr_db.clone();
                        let gitignore_obj_cloned = gitignore_builder_obj.clone();
                        async move {
                            let mut server = Server {
                                state: State::Running,
                                curr_db: curr_db_clone,
                                state_db_handler: state_db_handler_clone,
                            };

                            server
                                ._iterate_through_workspace(
                                    entry_path_clone.clone(),
                                    gitignore_obj_cloned,
                                )
                                .await
                        }
                    });
                } else if Server::_is_valid_file(&entry_path_path) {
                    log!(Level::Info, "File is valid: {}", entry_path_path.display());
                    files_set.spawn({
                        async move { Server::_index_file(entry_path_path.clone()).await }
                    });
                }
            }

            while let Some(res) = files_set.join_next().await {
                let output_authordetails = res.unwrap();

                if output_authordetails.is_empty() {
                    continue;
                }

                // 🛠 Group by file path and update DB
                let mut grouped_by_file: HashMap<String, Vec<AuthorDetailsV2>> = HashMap::new();

                for line_number in output_authordetails.keys() {
                    let detail = output_authordetails.get(line_number).unwrap();
                    grouped_by_file
                        .entry(detail.origin_file_path.clone())
                        .or_default()
                        .push(detail.clone());
                }

                for (origin_file_path, details_vec) in grouped_by_file {
                    if details_vec.is_empty() {
                        continue;
                    }
                    let db = self.curr_db.clone().unwrap();
                    let mut db_locked = db.lock().await;
                    let start_line_number = 0;
                    // Convert details_vec to HashMap<u32, AuthorDetailsV2>
                    let details_vec_map: HashMap<u32, AuthorDetailsV2> = details_vec
                        .iter()
                        .map(|detail| (detail.line_number as u32, detail.clone()))
                        .collect();
                    db_locked.append_to_db(
                        &origin_file_path,
                        start_line_number,
                        details_vec_map.clone(),
                    );
                    db_locked.store();
                    final_authordetails.extend(
                        details_vec
                            .into_iter()
                            .map(|detail| (detail.line_number as u32, detail)),
                    );
                }
            }
        } else if Server::_is_valid_file(path) {
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
        final_authordetails
    }

    pub async fn start_file(&mut self, metadata: &mut DBMetadata, file_path: Option<String>) {
        // Only index the given file and do no more than that.
        if file_path.is_none() {
            log!(Level::Error, "No file path provided to index.");
            return;
        }
        let file_path_str = file_path.clone().unwrap();
        let file_path_buf = PathBuf::from(file_path_str);
        let file_path_path = file_path_buf.as_path();
        if Server::_is_valid_file(file_path_path) {
            let workspace_path = &metadata.workspace_path;
            let db = DB {
                folder_path: workspace_path.clone(),
                ..Default::default()
            };
            let curr_db: Arc<Mutex<DB>> = Arc::new(db.into());
            curr_db
                .lock()
                .await
                .init_db(workspace_path.as_str(), None, false);
            let mut server = Server::new(State::Dead, DBHandler::new(metadata.clone()));
            server.init_server(curr_db);

        //     let out = Server::_index_file(file_path_buf.clone()).await;
        //     let db = server.curr_db.clone().unwrap();
        //     let mut db_locked = db.lock().await;
        //     let start_line_number = 0;
        //     println!(
        //         "Indexing file: {} with {} lines",
        //         file_path_buf.display(),
        //         out.len()
        //     );
        //     db_locked.append_to_db(&out[&0].origin_file_path, start_line_number, out.clone());
        //     db_locked.store();
        // }
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
        // In case we are attempging to index subfolders - do NOT cleanup
        // the DB.
        let mut cleanup: bool = true;
        if !metadata.folders_to_index.is_empty() {
            cleanup = false;
        }
        let curr_db: Arc<Mutex<DB>> = Arc::new(db.into());
        curr_db
            .lock()
            .await
            .init_db(workspace_path.as_str(), None, cleanup);
        let mut server = Server::new(State::Dead, DBHandler::new(metadata.clone()));
        server.init_server(curr_db);
        // Initialize a gitignore builder:
        let mut gitignore_builder = GitignoreBuilder::new(workspace_path_buf.clone());
        gitignore_builder.add(".gitignore");
        let gitignore = gitignore_builder.build();
        let mut gitignore_builder_obj: Option<Gitignore> = None;
        if gitignore.is_ok() {
            gitignore_builder_obj = Some(gitignore.unwrap());
        }

        if !self.state_db_handler.metadata.folders_to_index.is_empty() {
            // If subfolders are provided - just index them.
            for subfolder in self.state_db_handler.metadata.folders_to_index.iter() {
                let subfolder_path = PathBuf::from(format!("{}/{}", workspace_path, subfolder));
                if subfolder_path.exists() {
                    server
                        ._iterate_through_workspace(subfolder_path, gitignore_builder_obj.clone())
                        .await;
                } else {
                    println!("Subfolder does not exist: {}", subfolder);
                    log!(Level::Error, "Subfolder does not exist: {}", subfolder);
                }
            }
        } else {
            let _ = server
                ._iterate_through_workspace(workspace_path_buf.clone(), gitignore_builder_obj)
                .await;
        }
    }

    pub async fn handle_server(
        &mut self,
        workspace_path: &str,
        file_path: Option<String>,
        start_number: Option<usize>,
        end_number: Option<usize>,
        request_type: Option<RequestTypeOptions>,
        indexing_optional_folders: Option<Vec<String>>,
    ) {
        if request_type.is_some()
            && request_type.clone().unwrap() == RequestTypeOptions::ListSubdirs
        {
            git_command_algo::print_all_valid_directories(
                workspace_path.to_string(),
                Some(String::from(".gitignore")),
            );
            return;
        }
        // this will initialise any required states
        self.state_db_handler.init(workspace_path);
        self.state_db_handler.metadata.folders_to_index =
            indexing_optional_folders.unwrap_or(vec![]);
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
            self.curr_db
                .clone()
                .unwrap()
                .lock()
                .await
                .query(
                    file_path.clone().unwrap(),
                    start_number.unwrap(),
                    end_number.unwrap(),
                )
                .await;
        } else if request_type.is_some()
            && request_type.unwrap() == RequestTypeOptions::Descriptions
        {
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
            self.curr_db
                .clone()
                .unwrap()
                .lock()
                .await
                .query_descriptions(
                    file_path.clone().unwrap(),
                    start_number.unwrap(),
                    end_number.unwrap(),
                )
                .await;
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
                .handle_server(args.folder_path.as_str(), args.file, None, None, None, None)
                .await;
        }
        RequestTypeOptions::Author => {
            server
                .handle_server(args.folder_path.as_str(), args.file, None, None, None, None)
                .await;
        }
        RequestTypeOptions::Index => {
            // If indexing is mentioned, check if any subfolders are requested:
            let mut subfolders: Option<Vec<String>> = None;
            if args.index_subfolder.is_some() {
                // These will be comma separated paths.
                let subfolder_str = args.index_subfolder.unwrap();
                let subfolder_vec: Vec<String> =
                    subfolder_str.split(',').map(|s| s.to_string()).collect();
                subfolders.replace(subfolder_vec);
            }
            server
                .handle_server(
                    args.folder_path.as_str(),
                    None,
                    None,
                    None,
                    None,
                    subfolders,
                )
                .await;
        }
        RequestTypeOptions::IndexFile => {
            todo!("Indexing a single file is not supported yet.");
            // TODO: @krshrimali - fix this and re-enable.
            // server
            //     .handle_server(
            //         args.folder_path.as_str(),
            //         args.file,
            //         None,
            //         None,
            //         Some(RequestTypeOptions::IndexFile),
            //         None,
            //     )
            //     .await;
        }
        RequestTypeOptions::Query => {
            server
                .handle_server(
                    args.folder_path.as_str(),
                    args.file,
                    args.start_number,
                    args.end_number,
                    Some(RequestTypeOptions::Query),
                    None,
                )
                .await;
        }
        RequestTypeOptions::Descriptions => {
            server
                .handle_server(
                    args.folder_path.as_str(),
                    args.file,
                    args.start_number,
                    args.end_number,
                    Some(RequestTypeOptions::Descriptions),
                    None,
                )
                .await;
        }
        RequestTypeOptions::ListSubdirs => {
            // Just prints the subdirs to stdout
            server
                .handle_server(
                    args.folder_path.as_str(),
                    None,
                    None,
                    None,
                    Some(RequestTypeOptions::ListSubdirs),
                    None,
                )
                .await;
        }
    };
    Ok(())
}
