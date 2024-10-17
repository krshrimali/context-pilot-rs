mod algo_loc;
mod authordetails_impl;
mod config;
mod config_impl;
mod contextgpt_structs;
mod db;
mod git_command_algo;

use crate::{algo_loc::perform_for_whole_file, db::DB};
use async_recursion::async_recursion;
use contextgpt_structs::{AuthorDetails, Cli, RequestTypeOptions};
use std::{
    path::{Path, PathBuf},
    str::FromStr,
    sync::{Arc, Mutex},
};
use structopt::StructOpt;

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

    async fn _index_file(file: PathBuf) -> Vec<AuthorDetails> {
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
        output_author_details
        // Now extract output string from the output_author_details.
        // extract_string_from_output(output_author_details, /*is_author_mode=*/ false)
    }

    #[async_recursion]
    async fn _iterate_through_workspace(
        workspace_path: PathBuf,
        config_file_path: PathBuf,
    ) -> Vec<AuthorDetails> {
        let mut set: task::JoinSet<()> = task::JoinSet::new();
        let mut files_set: task::JoinSet<Vec<AuthorDetails>> = task::JoinSet::new();

        let path = Path::new(&workspace_path);
        let mut final_authordetails: Vec<AuthorDetails> = Vec::new();

        if path.is_dir() {
            // iterate through the directory and start indexing all the files
            for entry in path
                .read_dir()
                .unwrap_or_else(|_| panic!("failed reading directory {}", path.display()))
            {
                let entry_path = entry.unwrap().path();
                if entry_path.is_dir() {
                    // FIXME: This is a case of having a sub-directory
                    files_set.spawn(Server::_iterate_through_workspace(
                        entry_path.clone(),
                        config_file_path.clone(),
                    ));
                } else {
                    log!(Level::Info, "File is valid: {}", entry_path.display());

                    files_set.spawn(Server::_index_file(entry_path));
                }
            }

            while let Some(res) = files_set.join_next().await {
                let output_authordetails = res.unwrap();
                final_authordetails.extend(output_authordetails);
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
                let output = Server::_index_file(path.to_path_buf()).await;
                return output;
            } else {
                log!(Level::Warn, "File is not valid: {}", path.display());
            }
        }

        final_authordetails
    }

    pub async fn start_file(&mut self, metadata: &mut DBMetadata, file_path: Option<PathBuf>) {
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

        let db = DB {
            folder_path: workspace_path.clone(),
            ..Default::default()
        };
        let curr_db: Arc<Mutex<DB>> = Arc::new(db.into());
        curr_db.lock().unwrap().init_db(workspace_path.as_str());
        let output = Server::_iterate_through_workspace(
            workspace_path_buf.clone(),
            workspace_path_buf.clone(), // unused
        )
        .await;

        let origin_file_path = metadata.workspace_path.clone();
        let start_line_number = 0;
        curr_db
            .lock()
            .unwrap()
            .append(&origin_file_path, start_line_number, output.clone());
        curr_db.lock().unwrap().store();
    }

    pub async fn handle_server(&mut self, workspace_path: &str, file_path: Option<PathBuf>) {
        // this will initialise any required states
        self.state_db_handler.init(workspace_path);

        let mut metadata: DBMetadata = self.state_db_handler.get_current_metadata();

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
                        if workspace_path.is_empty() {panic!("no workspace path passed")}
                        self.start_indexing(&mut metadata).await;
                    },
                    _ => {
                        self.start_file(&mut metadata, file_path).await;
                    }
                }
            }
        }
    }

    fn cont(&mut self) {
        // Start from the line number and file that you were at and continue indexing
        todo!();
    }
}

#[tokio::main]
async fn main() -> CliResult {
    let args = Cli::from_args();

    env_logger::init();
    let mut server = Server {
        state: State::Dead,
        state_db_handler: DBHandler {
            db: DB::default(),
            metadata: DBMetadata::default(),
        },
    };

    // TODO: Add support for config file.
    // let config_obj: config_impl::Config = config_impl::read_config(config::CONFIG_FILE_NAME);
    let mut file_path: Option<PathBuf> = None;
    if args.file.is_some() {
        file_path = PathBuf::from_str(args.file.unwrap().as_str()).unwrap().into();
    }

    match args.request_type {
        RequestTypeOptions::File => {
            server
                .handle_server(args.folder_path.as_str(), file_path)
                .await;
        }
        RequestTypeOptions::Author => {
            server
                .handle_server(args.folder_path.as_str(), file_path)
                .await;
        }
        RequestTypeOptions::Index => {
            server.handle_server(args.folder_path.as_str(), None).await;
        }
    };
    Ok(())
}
