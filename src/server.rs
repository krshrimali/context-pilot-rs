use crate::{algo_loc::perform_for_whole_file, db::DB};
use std::path::{Path, PathBuf};

use quicli::prelude::log::{log, Level};

use crate::{config, config_impl};

#[derive(Default, Debug, Eq, PartialEq)]
pub enum State {
    Starting, // Indexing alr
    Running,  // Indexing finished
    #[default]
    Stopped, // Server is not running
    Failed,   // On any failure, but the server is still running
}

pub struct DBHandler {
    db: DB,
}

impl DBHandler {
    pub fn init(&mut self, folder_path: &str) {
        self.db = DB {
            folder_path: folder_path.to_string(),
            ..Default::default()
        };
    }

    pub fn retry(&mut self) {
        todo!();
    }

    pub fn get_current_metadata(&mut self) -> DBMetadata {
        DBMetadata::default()
    }

    pub fn start(&mut self, metadata: &DBMetadata) {
        // this should ideally start the DB server
        // DB Server and the other server should be kept separate
        // this should not be async though - as we'll really want this to finish before it finishes
        self.db.init_db(&metadata.workspace_path);
    }
}

pub struct Server {
    state: State,
    state_db_handler: DBHandler,
}

#[derive(Default, Debug)]
pub struct DBMetadata {
    state: State,
    workspace_path: String,
    curr_progress: i64, // file index you're at OR percentage done
    total_count: i64,   // how many files are indexing
}

impl DBMetadata {}

impl Server {
    fn _is_valid_file(&self, file: &Path) -> bool {
        if file.exists() && file.is_file() {
            // not optimising one liners here for debugging later on
            log!(Level::Debug, "File exists: {}", file.display());
            return true;
        }
        false
    }

    fn _index_file(&self, file: &PathBuf) {
        let file_path = file.to_str().unwrap();
        let mut db_obj = DB {
            folder_path: "".to_string(),
            ..Default::default()
        };

        // Read the config file and pass defaults
        let config_obj: config_impl::Config = config_impl::read_config(config::CONFIG_FILE_NAME);

        db_obj.init_db(file_path);
        let output_str =
            perform_for_whole_file(file_path.to_string(), &mut db_obj, true, &config_obj);
        println!("output string: {output_str}");
    }

    async fn _iterate_through_workspace(&mut self, workspace_path: &PathBuf) {
        async fn _reiterate_workspace(entry: std::fs::DirEntry) {
            println!("workspace path: {}", entry.path().display());
            // let entry = entry.unwrap();
            // let path = entry.path();
            // if path.is_dir() {
            //     tasks.push(tokio::spawn(_reiterate_workspace(&path)));
            // } else {
            // check if the file is valid
            // if it is, then index it
            // if it's not, then just raise a warning
            // if _is_valid_file(&path) {
            //     _index_file(&path);
            // } else {
            //     log!(Level::Warn, "File is not valid: {}", path.display());
            // }
        }

        let path = Path::new(&workspace_path);

        let mut tasks = vec![];

        if path.is_dir() {
            // iterate through the directory and start indexing all the files
            for entry in path
                .read_dir()
                .unwrap_or_else(|_| panic!("failed reading directory {}", path.display()))
            {
                tasks.push(tokio::spawn(_reiterate_workspace(entry.unwrap())));
            }

            let mut outputs = vec![];
            for task in tasks {
                outputs.push(task.await.unwrap());
            }
        } else {
            // path is not a directory
            // in which case, you might just want to index it if it's a valid file - or else - just raise a warning
            if self._is_valid_file(path) {
                self._index_file(&path.to_path_buf());
            } else {
                log!(Level::Warn, "File is not valid: {}", path.display());
            }
        }
    }

    pub async fn start(&mut self, metadata: &mut DBMetadata) {
        // start the server for the given workspace
        // todo: see if you just want to pass the workspace path and avoiding passing the whole metadata here
        let workspace_path = &metadata.workspace_path;

        // the server will start going through all the "valid" files in the workspace and will index them
        // defn of valid: files that satisfy .gitignore check (if exists and not disabled in the config)
        // and files that are not in the ignore list if provided in the config

        let workspace_path_buf = PathBuf::from(workspace_path);
        self._iterate_through_workspace(&workspace_path_buf);
    }

    pub fn handle_server(&mut self, workspace_path: &str) {
        // this will initialise any required states
        self.state_db_handler.init(workspace_path);

        let mut metadata: DBMetadata = self.state_db_handler.get_current_metadata();

        if metadata.workspace_path == workspace_path {
            if metadata.state == State::Running {
                // the server is already running
                // do nothing - let the indexing continue
                println!("[CONTINUING] Progress: {}", metadata.curr_progress);
            } else if metadata.state == State::Starting {
                // the server is not in running state -> for state of it's attempting to start -> let it finish and then see if it was successful
                // TODO: @krshrimali
                self.state_db_handler.start(&metadata);
                self.start(&mut metadata);
            } else if metadata.state == State::Failed {
                // in case of failure though, ideally it would have been alr handled by other process -> but in any case, starting from here as well to just see how it works out
                // I'm in the favor of not restarting in case of failure from another process though
                self.state_db_handler.retry();
            } else if metadata.state == State::Stopped {
                self.state_db_handler.start(&metadata);
            }
            // in case the metadata workspace path matches with the input and the server is already running -> don't do indexing
        }
    }

    fn cont(&mut self) {
        // Start from the line number and file that you were at and continue indexing
        todo!();
    }
}
