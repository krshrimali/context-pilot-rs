use crate::contextgpt_structs::AuthorDetails;

// AuthorDetails is a struct that contains the details of the author of the commit.
// It is used to store the details of the author of the commit.
impl AuthorDetails {
    pub fn serialize_from_str(
        input_str: String,
        commit_hash: String,
        file_path: &str,
        context_file_paths: Vec<String>,
        end_line_number: usize,
    ) -> AuthorDetails {
        let parts: Vec<&str> = input_str.split_whitespace().collect();
        if parts.is_empty() {
            return AuthorDetails::default(); // or log and return dummy
        }

        // println!("parts: {:?}", parts);

        let line_number = match parts.last().unwrap().parse::<usize>() {
            Ok(num) => num,
            Err(_) => {
                // eprintln!("Error parsing line number from input string");
                return AuthorDetails::default(); // or log and return dummy
            }
        };

        // Try to extract author name from the tail end.
        let author_raw = parts.iter().rev().take(5).cloned().collect::<Vec<_>>();
        let author_original_name = author_raw
            .iter()
            .rev()
            .cloned()
            .collect::<Vec<_>>()
            .join(" ");

        AuthorDetails {
            commit_hash,
            author_full_name: author_original_name,
            origin_file_path: file_path.to_string(),
            line_number,
            contextual_file_paths: context_file_paths,
            end_line_number,
        }
    }
}
