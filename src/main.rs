mod algo_loc;
mod authordetails_impl;
mod config;
mod config_impl;
mod contextgpt_structs;
mod db;
mod git_command_algo;
// mod server;

// mod async_check;
use crate::{algo_loc::extract_string_from_output, algo_loc::perform_for_whole_file, db::DB};
use async_recursion::async_recursion;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use quicli::prelude::log::{log, Level};
use tokio::task;

#[derive(Default, Debug, Eq, PartialEq, Clone, Copy)]
pub enum State {
    Starting, // Indexing alr
    Running,  // Indexing finished
    #[default]
    Dead, // Server is not running
    Failed,   // On any failure, but the server is still running
}

pub struct DBHandler {
    db: DB,
    metadata: DBMetadata,
}

impl DBHandler {
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
        self.db = DB {
            folder_path: folder_path.to_string(),
            ..Default::default()
        };

        // iterate through folder_path and calculate total number of valid files
        // and set the metadata
        let total_valid_file_count = self._valid_file_count(folder_path);

        self.metadata = DBMetadata {
            // Initial state should be stopped or..?
            state: State::Dead,
            workspace_path: folder_path.to_string(),
            curr_progress: 0,
            total_count: total_valid_file_count,
        }
    }

    pub fn retry(&mut self) {
        todo!();
    }

    pub fn get_current_metadata(&mut self) -> DBMetadata {
        self.metadata.clone()
    }

    pub fn start(&mut self, metadata: &DBMetadata) {
        // this should ideally start the DB server
        // DB Server and the other server should be kept separate
        // this should not be async though - as we'll really want this to finish before it finishes
        println!(
            "Passing workspace path to init_db: {}",
            metadata.workspace_path
        );
    }
}

pub struct Server {
    state: State,
    // curr_db: Option<Arc<Mutex<DB>>>,
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
    fn _is_valid_file(file: &Path) -> bool {
        if file.exists() && file.is_file() {
            // not optimising one liners here for debugging later on
            log!(Level::Debug, "File exists: {}", file.display());
            return true;
        }
        false
    }

    async fn _index_file(file: PathBuf) -> String {
        // Don't make it write to the DB, write it atomically later.
        // For now, just store the output somewhere in the DB.
        let file_path = file.to_str().unwrap();
        // let mut db_obj = DB {
        //     folder_path: workspace_path.clone(),
        //     ..Default::default()
        // };
        // db_obj.init_db(workspace_path.as_str());

        // Read the config file and pass defaults
        let config_obj: config_impl::Config = config_impl::read_config(config::CONFIG_FILE_NAME);

        // curr_db.init_db(file_path);
        let output_author_details = perform_for_whole_file(file_path.to_string(), &config_obj);
        // Now extract output string from the output_author_details.
        let output_str =
            extract_string_from_output(output_author_details, /*is_author_mode=*/ false);
        // println!("output string: {output_str}");
        output_str
    }

    #[async_recursion]
    async fn _iterate_through_workspace(&mut self, workspace_path: PathBuf, _: PathBuf) {
        let mut set: task::JoinSet<()> = task::JoinSet::new();
        let mut files_set: task::JoinSet<String> = task::JoinSet::new();

        let path = Path::new(&workspace_path);

        if path.is_dir() {
            // iterate through the directory and start indexing all the files
            for entry in path
                .read_dir()
                .unwrap_or_else(|_| panic!("failed reading directory {}", path.display()))
            {
                let entry_path = entry.unwrap().path();
                if entry_path.is_dir() {
                    // FIXME: This is a case of having a sub-directory
                    // println!("ignoring directory: {}", entry_path.display());
                    // set.spawn(Server::_iterate_through_workspace(
                    //     // self,
                    //     entry_path.clone(),
                    //     default_folder_path.clone(),
                    // ));
                    println!("Continuing...");
                    continue;
                } else {
                    log!(Level::Info, "File is valid: {}", entry_path.display());

                    files_set.spawn(Server::_index_file(entry_path));
                }
            }

            let mut seen = HashSet::new();

            // for res in tasks_list.iter() {
            //     let idx = res.await.unwrap();
            //     seen.insert(idx.clone());
            //     log!(Level::Info, "Done with file: {}", idx);
            // }

            while let Some(res) = set.join_next().await {
                let idx = res.unwrap();
                seen.insert(idx.clone());
                // log!(Level::Info, "Done with file: {:?}", idx);
            }

            let mut files_seen = HashSet::new();
            while let Some(res) = files_set.join_next().await {
                let idx = res.unwrap();
                files_seen.insert(idx.clone());
                log!(Level::Info, "Done with file: {:?}", idx);
            }
        } else {
            // path is not a directory
            // in which case, you might just want to index it if it's a valid file - or else - just raise a warning
            if Server::_is_valid_file(path) {
                log!(
                    Level::Warn,
                    "File is valid but not in a sub-directory: {}",
                    path.display()
                );
                // Server::_index_file(
                //     path.to_path_buf(),
                //     workspace_path.to_str().unwrap().to_string(),
                // )
                // .await;
            } else {
                log!(Level::Warn, "File is not valid: {}", path.display());
            }
        }
    }

    pub async fn start(&mut self, metadata: &mut DBMetadata) {
        // start the server for the given workspace
        // TODO: see if you just want to pass the workspace path and avoiding passing the whole metadata here
        let workspace_path = &metadata.workspace_path;

        // the server will start going through all the "valid" files in the workspace and will index them
        // defn of valid: files that satisfy .gitignore check (if exists and not disabled in the config)
        // and files that are not in the ignore list if provided in the config

        let workspace_path_buf = PathBuf::from(workspace_path);
        println!("Now starting to iterate through the workspace...");
        // let curr_db = DB {
        //     folder_path: workspace_path.clone(),
        //     ..Default::default()
        // };

        let db = DB {
            folder_path: workspace_path.clone(),
            ..Default::default()
        };
        let curr_db: Arc<Mutex<DB>> = Arc::new(db.into());
        curr_db.lock().unwrap().init_db(workspace_path.as_str());
        self._iterate_through_workspace(workspace_path_buf.clone(), workspace_path_buf)
            .await
    }

    pub async fn handle_server(&mut self, workspace_path: &str) {
        // this will initialise any required states
        self.state_db_handler.init(workspace_path);

        let mut metadata: DBMetadata = self.state_db_handler.get_current_metadata();

        println!("metadata: {:?}", metadata);
        println!("workspace path: {}", workspace_path);

        let mut tasks = vec![];

        if metadata.workspace_path == workspace_path {
            if metadata.state == State::Running {
                // the server is already running
                // do nothing - let the indexing continue
                println!("[CONTINUING] Progress: {}", metadata.curr_progress);
            } else if metadata.state == State::Starting {
                // the server is not in running state -> for state of it's attempting to start -> let it finish and then see if it was successful
                // TODO: @krshrimali
                self.state_db_handler.start(&metadata);
                tasks.push(self.start(&mut metadata));
            } else if metadata.state == State::Failed {
                // in case of failure though, ideally it would have been alr handled by other process -> but in any case, starting from here as well to just see how it works out
                // I'm in the favor of not restarting in case of failure from another process though
                // TODO: Have a limit here, can't retry for infinite count.
                self.state_db_handler.retry();
            } else if metadata.state == State::Dead {
                println!("Starting server...");

                // should be blocking rn
                self.state_db_handler.start(&metadata);

                self.start(&mut metadata).await;
                println!("Done");
            }
            // in case the metadata workspace path matches with the input and the server is already running -> don't do indexing
        }
    }

    fn cont(&mut self) {
        // Start from the line number and file that you were at and continue indexing
        todo!();
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let mut server = Server {
        state: State::Dead,
        state_db_handler: DBHandler {
            db: DB::default(),
            metadata: DBMetadata::default(),
        },
    };

    server
        .handle_server("/home/krshrimali/Documents/Projects/context-pilot-rs")
        .await;
}
