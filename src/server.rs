#[derive(Default, Debug)]
pub enum State {
    Starting,
    Running,
    #[default] Stopped,
    Failed,
}

pub struct DBHandler {}

impl DBHandler {
    pub fn retry() {
        return;
    }

    pub fn get_current_metadata() -> DBMetadata {
        return DBMetadata::default();
    }

    pub fn start() {
        return;
    }
}

pub struct Server {
    state: State,
    state_db_handler: DBHandler,
}

#[derive(Default)]
pub struct DBMetadata {
    state: State,
    workspace_path: String,
    curr_progress: i64,
    total_count: i64,
}

impl Server {
    pub fn start_server(&mut self, workspace_path: &str) {
        // this will initialise any required states
        self.state_db_handler.init();

        let metadata: DBMetadata = self.state_db_handler.get_current_metadata();

        if metadata.workspace_path == workspace_path {
            if metadata.state == State::Running {
                // the server is already running
                // do nothing - let the indexing continue
                println!("[CONTINUING] Progress: {}", metadata.curr_progress);
            } else if metadata.state == State::Starting {
                // the server is not in running state -> for state of it's attempting to start -> let it finish and then see if it was successful
                // TODO: @krshrimali
            } else if metadata.state == State::Failed {
                // in case of failure though, ideally it would have been alr handled by other process -> but in any case, starting from here as well to just see how it works out
                // I'm in the favor of not restarting in case of failure from another process though
                self.state_db_handler.retry();
            } else if metadata.store == State::Stopped {
                self.state_db_handler.start();
            }
            // in case the metadata workspace path matches with the input and the server is already running -> don't do indexing
        }
    }

    fn continue(&mut self) {
        // Start from the line number and file that you were at and continue indexing
    }
}
