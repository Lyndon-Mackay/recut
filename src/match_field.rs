use crate::field::split_line_quotes;
use std::collections::HashMap;
pub fn parse_match_indices(
    match_str: &str,
    input_line: &str,
    delimiter: &str,
) -> (Vec<String>, Vec<usize>) {
    let match_split = split_line_quotes(match_str, ",");

    let line_split = split_line_quotes(input_line, delimiter);

    let mut line_hash = HashMap::new();

    for (i, line) in line_split.into_iter().enumerate() {
        line_hash.entry(line).or_insert(i);
    }

    let mut indices = Vec::with_capacity(match_split.len());
    let mut first_line_split = Vec::with_capacity(match_split.len());

    for line in match_split.into_iter() {
        if let Some(k) = line_hash.get(&line) {
            indices.push(*k);
            first_line_split.push(line);
        }
    }
    println!(
        "{:?} {:?},{:?} ,{:?}",
        first_line_split, indices, line_hash, delimiter
    );
    (first_line_split, indices)
}
