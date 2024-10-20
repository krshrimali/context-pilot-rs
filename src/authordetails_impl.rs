use crate::contextgpt_structs::AuthorDetails;

impl AuthorDetails {
    pub fn serialize_from_str(
        input_str: String,
        commit_hash: String,
        file_path: &str,
        context_file_paths: Vec<String>,
        end_line_number: usize,
    ) -> AuthorDetails {
        let mut author_str_split: Vec<&str> = input_str.split(' ').collect();
        author_str_split.reverse();
        let mut author_name: String = "".to_string();
        let mut count: usize = 0;
        for (_, each_split) in author_str_split.iter().enumerate() {
            if each_split.is_empty() {
                continue;
            }
            count += 1;
            if count >= 5 {
                author_name += each_split;
                author_name += " ";
            }
        }
        author_name = author_name.trim_end().to_string();
        let mut names: Vec<&str> = author_name.split(' ').collect();
        names.reverse();
        let author_original_name = names.join(" ");
        let author_details = AuthorDetails {
            commit_hash,
            author_full_name: author_original_name,
            origin_file_path: file_path.to_string(),
            line_number: author_str_split.first().unwrap().parse::<usize>().unwrap(),
            contextual_file_paths: context_file_paths,
            end_line_number,
        };
        author_details
    }
}
