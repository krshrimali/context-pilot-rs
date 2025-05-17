pub fn levenshtein(a: &str, b: &str) -> usize {
    let mut costs: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut last_cost = i;
        costs[0] = i + 1;
        for (j, cb) in b.chars().enumerate() {
            let current_cost = costs[j + 1];
            if ca == cb {
                costs[j + 1] = last_cost;
            } else {
                costs[j + 1] = 1 + std::cmp::min(std::cmp::min(current_cost, last_cost), costs[j]);
            }
            last_cost = current_cost;
        }
    }
    *costs.last().unwrap()
}

// Compare a single line vs another line and see if they are similar (> some threshold).
fn is_similar(line1: &str, line2: &str, threshold: usize) -> bool {
    // FYI: Levenshtein distance is a measure of the difference between two sequences.
    // It is calculated as the minimum number of single-character edits (insertions, deletions, or
    // substitutions) required to change one word into the other.
    // The distance is a non-negative integer, and the smaller the distance, the more similar the
    // two sequences are.
    // For example, the Levenshtein distance between "kitten" and "sitting" is 3, as it takes three
    // operations to transform one into the other:
    let distance = levenshtein(line1, line2);
    distance <= threshold
}
