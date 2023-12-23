use crate::config;

use serde::Deserialize;
use serde::Serialize;
use std::fs::File;
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub file_count_threshold: usize,
    pub commit_hashes_threshold: usize, // TODO: Have a default value here later on
}

impl Default for Config {
    fn default() -> Self {
        Config {
            file_count_threshold: config::OUTPUT_COUNT_THRESHOLD,
            commit_hashes_threshold: config::LAST_MANY_COMMIT_HASHES,
        }
    }
}

pub fn read_config(config_file_name: &str) -> Config {
    let home_dir_path: PathBuf = simple_home_dir::home_dir().unwrap_or_default();
    let joined_path = home_dir_path.join(config_file_name);
    let config_file = File::open(joined_path.clone());

    match config_file {
        Ok(_) => {
            let mut config_stream = serde_json::Deserializer::from_reader(config_file.unwrap());
            // TODO: Fix the error handling and ensure users are able to watch logs for this
            let config = Config::deserialize(&mut config_stream)
                .expect("Expected config to be deserialized, probably the format is wrong");
            config
        }
        Err(_) => {
            // TODO: Add a log that you are using defaults
            Config::default()
        }
    }
}

pub fn trim_result(inp: String, threshold: usize) -> String {
    // number of words in the string should not cross threshold
    // just trim the string
    let mut final_str: String = "".to_string();
    let mut count: usize = 0;
    for inp_word in inp.split(',') {
        final_str.push_str(inp_word);
        count += 1;
        if count >= threshold {
            break;
        }
        final_str.push(',');
    }
    final_str
}

mod test {
    use super::*;

    #[test]
    fn test_trim_result() {
        let inp = "a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z".to_string();
        let threshold = 10;
        let out = trim_result(inp, threshold);
        assert_eq!(out, "a,b,c,d,e,f,g,h,i,j,k");
    }

    #[test]
    fn test_trim_result_from_config() {
        let inp = "a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z".to_string();
        let config_obj = Config {
            file_count_threshold: 10,
            commit_hashes_threshold: 10,
        };
        let out = trim_result(inp, config_obj.file_count_threshold);
        assert_eq!("a,b,c,d,e,f,g,h,i,j,k", out);
    }
}
