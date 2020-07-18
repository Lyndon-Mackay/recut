use crate::field::{split_line_quotes, split_line_regex_quotes};
use regex::{Regex, RegexSetBuilder};

use crate::error;
use error::RecutError;

enum DelimiterType<'a> {
    String(&'a str),
    Regex(&'a Regex),
}

pub fn parse_match_indices(
    match_str: &str,
    input_line: &str,
    delimiter: &str,
) -> Result<(Vec<String>, Vec<usize>), RecutError> {
    parse_match(match_str, input_line, &DelimiterType::String(delimiter))
}

pub fn parse_match_indices_regex(
    match_str: &str,
    input_line: &str,
    delimiter: &Regex,
) -> Result<(Vec<String>, Vec<usize>), RecutError> {
    parse_match(match_str, input_line, &DelimiterType::Regex(delimiter))
}

fn parse_match(
    match_str: &str,
    input_line: &str,
    delim: &DelimiterType,
) -> Result<(Vec<String>, Vec<usize>), RecutError> {
    let match_split = split_line_quotes(match_str, ",");

    let line_split = match delim {
        DelimiterType::String(delimiter) => split_line_quotes(input_line, delimiter),
        DelimiterType::Regex(delimiter) => split_line_regex_quotes(input_line, delimiter),
    };

    let set = RegexSetBuilder::new(match_split.iter())
        .case_insensitive(true)
        .build()?;

    let mut indices = Vec::with_capacity(match_split.len());
    let mut first_line_split = Vec::with_capacity(match_split.len());

    for (i, line) in line_split.into_iter().enumerate() {
        if set.is_match(&line) {
            indices.push(i);
            first_line_split.push(line);
        }
    }

    Ok((first_line_split, indices))
}
