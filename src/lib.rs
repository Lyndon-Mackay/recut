extern crate pest;

use field::{split_line_quotes, split_line_regex_quotes};
use fs::File;
use io::{stdin, BufRead, BufReader};
use match_field::{parse_match_indices, parse_match_indices_regex};
use pest::Parser;
use range::{parse_indices, BeginRange, EndRange, UnExpandedIndices};
use regex::Regex;
use std::{
    collections::{BTreeMap, HashMap},
    error, fmt, fs, io,
    num::ParseIntError,
};

#[macro_use]
extern crate pest_derive;

mod field;
mod match_field;
mod range;

#[derive(Debug)]
pub enum IoType {
    FromStdIn,
    FromFile(String),
}

#[derive(Clone, Debug)]
pub struct RangeDelimiter<'a> {
    locations: &'a str,
    delimiter: &'a str,
}

#[derive(Clone, Debug)]
pub enum CutType<'a> {
    Bytes(&'a str, bool),
    Characters(&'a str),
    FieldsInferDelimiter(&'a str),
    FieldsRegexDelimiter(RangeDelimiter<'a>),
    FieldsStringDelimiter(RangeDelimiter<'a>),
    MatchesInferDelimiter(&'a str),
    MatchesRegexDelimiter(RangeDelimiter<'a>),
    MatchesStringDelimiter(RangeDelimiter<'a>),
}

impl RangeDelimiter<'_> {
    pub fn new<'a>(locations: &'a str, delimiter: &'a str) -> RangeDelimiter<'a> {
        RangeDelimiter {
            locations,
            delimiter,
        }
    }
}
#[derive(Parser)]
#[grammar = "input.pest"]
pub struct InputParser;

#[derive(Clone, Debug)]
enum RangeParseError {
    IntError(ParseIntError),
}

impl fmt::Display for RangeParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            //RangeParseError::RangeError(i, j) => write!(f, "{} should be less then {}", i, j),
            // This is a wrapper, so defer to the underlying types' implementation of `fmt`.
            RangeParseError::IntError(ref e) => e.fmt(f),
        }
    }
}

impl error::Error for RangeParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            RangeParseError::IntError(ref e) => Some(e),
            //RangeParseError::RangeError(_, _) => None,
        }
    }
}
impl From<ParseIntError> for RangeParseError {
    fn from(err: ParseIntError) -> RangeParseError {
        RangeParseError::IntError(err)
    }
}

pub fn cut(input: IoType, cut_type: CutType) -> Result<(), io::ErrorKind> {
    let input: Box<dyn BufRead> = match input {
        IoType::FromStdIn => Box::new(BufReader::new(stdin())),
        IoType::FromFile(file_name) => {
            let file = File::open(file_name).unwrap();

            let reader = BufReader::new(file);

            Box::new(BufReader::new(reader))
        }
    };

    match cut_type {
        CutType::Bytes(range, split) => {
            let parsed_indices = parse_indices(range).unwrap();
            print_by_bytes(input, split, &parsed_indices)
        }
        CutType::Characters(range) => {
            let parsed_indices = parse_indices(range).unwrap();
            print_by_character(input, &parsed_indices)
        }
        CutType::FieldsInferDelimiter(range) => {
            let parsed_indices = parse_indices(range).unwrap();
            print_infer_regex(input, &parsed_indices)
        }
        CutType::FieldsRegexDelimiter(range) => {
            let parsed_indices = parse_indices(range.locations).unwrap();
            print_by_regex(input, &range.delimiter, &parsed_indices)
        }
        CutType::FieldsStringDelimiter(range) => {
            let parsed_indices = parse_indices(range.locations).unwrap();
            print_by_string_delimiter(input, &range.delimiter, &parsed_indices)
        }
        CutType::MatchesInferDelimiter(range) => {
            print_match_infer_regex(input, range);
        }
        CutType::MatchesRegexDelimiter(range) => {
            print_match_regex_delimiter(input, range.delimiter, range.locations)
        }
        CutType::MatchesStringDelimiter(range) => {
            print_match_string_delimiter(input, range.delimiter, range.locations)
        }
    }

    Ok(())
}

fn print_by_character(input_buffer: Box<dyn BufRead>, input_indices: &[UnExpandedIndices]) {
    for line in input_buffer.lines() {
        print_line_by_character(&line.unwrap(), &input_indices)
    }
}

fn print_line_by_character(input_line: &str, input_indices: &[UnExpandedIndices]) {
    let length = input_line.chars().count();
    let (sorted_indices, expanded_indices) = expand_indices(input_indices, length);
    let first_index = *sorted_indices.first().unwrap();

    let last_index = *sorted_indices.last().unwrap();

    let take_length = last_index + 1;

    let char_map = input_line
        .char_indices()
        .skip(first_index)
        .take(take_length)
        .collect::<HashMap<_, _>>();

    let mut print_string = String::new();
    for print_index in &expanded_indices {
        print_string.push(*char_map.get(print_index).unwrap());
    }
    println!("{}", print_string);
}

fn print_by_bytes(
    input_buffer: Box<dyn BufRead>,
    splits_allowed: bool,
    input_indices: &[UnExpandedIndices],
) {
    for line in input_buffer.lines() {
        print_line_by_bytes(&line.unwrap(), splits_allowed, &input_indices)
    }
}

fn print_line_by_bytes(input_line: &str, splits_alowed: bool, input_indices: &[UnExpandedIndices]) {
    let length = input_line.bytes().count();

    let (sorted_indices, expanded_indices) = expand_indices(input_indices, length);

    let first_index = *sorted_indices.first().unwrap();

    let last_index = *sorted_indices.last().unwrap();

    let take_length = last_index + 1;

    let byte_map = input_line
        .bytes()
        .enumerate()
        .skip(first_index)
        .take(take_length)
        .collect::<HashMap<_, _>>();

    let mut print_bytes = Vec::with_capacity(length);

    for print_index in expanded_indices {
        print_bytes.push(*byte_map.get(&print_index).unwrap());
    }

    let print_string = String::from_utf8_lossy(print_bytes.as_slice());

    if splits_alowed {
        println!("{}", print_string);
    } else {
        println!("{}", print_string.trim_end_matches("�"))
    }
}

fn print_by_string_delimiter(
    input_buffer: Box<dyn BufRead>,
    delimiter: &str,
    input_indices: &[UnExpandedIndices],
) {
    for line in input_buffer.lines() {
        let split_line = split_line_quotes(&line.unwrap(), delimiter);

        print_line_delimited(&split_line, &input_indices)
    }
}
fn print_by_regex(
    input_buffer: Box<dyn BufRead>,
    delimiter: &str,
    input_indices: &[UnExpandedIndices],
) {
    let regex_delim = Regex::new(delimiter).unwrap();

    for line in input_buffer.lines() {
        let split_line = split_line_regex_quotes(&line.unwrap(), &regex_delim);
        print_line_delimited(&split_line, &input_indices)
    }
}
fn print_line_delimited(split_line: &[String], input_indices: &[UnExpandedIndices]) {
    let length = split_line.len();
    let (sorted_indices, expanded_indices) = expand_indices(input_indices, length);
    let first_index = *sorted_indices.first().unwrap();

    let last_index = *sorted_indices.last().unwrap();

    let take_length = last_index + 1;

    let split_map = split_line
        .into_iter()
        .enumerate()
        .skip(first_index)
        .take(take_length)
        .collect::<HashMap<_, _>>();

    let mut print_string = Vec::with_capacity(length);

    for print_index in expanded_indices {
        let next = split_map.get(&print_index).unwrap().to_owned().to_owned();
        print_string.push(next)
    }

    println!("{}", print_string.join(""));
}

fn print_infer_regex(mut input_buffer: Box<dyn BufRead>, input_indices: &[UnExpandedIndices]) {
    let mut line = String::new();
    input_buffer.read_line(&mut line).unwrap();
    let delimiter = infer_delimiter(&line);

    let split_line = split_line_quotes(&line, &delimiter);
    print_line_delimited(&split_line, &input_indices);
    for line in input_buffer.lines() {
        let split_line = split_line_quotes(&line.unwrap(), &delimiter);
        print_line_delimited(&split_line, &input_indices);
    }
}

fn print_match_infer_regex(mut input_buffer: Box<dyn BufRead>, match_str: &str) {
    let mut line = String::new();
    input_buffer.read_line(&mut line).unwrap();
    let delimiter = infer_delimiter(&line);

    let (split_line, input_indices) = parse_match_indices(match_str, &line, &delimiter);

    println!("{}", split_line.join(","));
    for line in input_buffer.lines() {
        let split_line = split_line_quotes(&line.unwrap(), &delimiter);
        print_line_match_delimited(&split_line, &input_indices);
    }
}
fn print_line_match_delimited(split_line: &[String], input_indices: &[usize]) {
    let mut print_string = Vec::with_capacity(input_indices.len());

    let split_indices = split_line
        .into_iter()
        .enumerate()
        .collect::<HashMap<_, _>>();

    for i in input_indices {
        let next = split_indices.get(&i).unwrap().to_owned().to_owned();
        print_string.push(next);
    }

    println!("{}", print_string.join(","));
}

fn print_match_string_delimiter(
    mut input_buffer: Box<dyn BufRead>,
    delimiter: &str,
    match_str: &str,
) {
    let mut line = String::new();
    input_buffer.read_line(&mut line).unwrap();

    let (split_line, input_indices) = parse_match_indices(match_str, &line, &delimiter);

    println!("{}", split_line.join(","));
    for line in input_buffer.lines() {
        let split_line = split_line_quotes(&line.unwrap(), &delimiter);
        print_line_match_delimited(&split_line, &input_indices);
    }
}

fn print_match_regex_delimiter(
    mut input_buffer: Box<dyn BufRead>,
    delimiter: &str,
    match_str: &str,
) {
    let regex = Regex::new(delimiter).unwrap();
    let mut line = String::new();
    input_buffer.read_line(&mut line).unwrap();

    let (split_line, input_indices) = parse_match_indices_regex(match_str, &line, &regex);

    println!("{}", split_line.join(","));
    for line in input_buffer.lines() {
        let split_line = split_line_regex_quotes(&line.unwrap(), &regex);
        print_line_match_delimited(&split_line, &input_indices);
    }
}

fn infer_delimiter(input_line: &str) -> String {
    let parse_result = InputParser::parse(Rule::input, input_line).unwrap();

    let mut potential_delimiters = BTreeMap::new();
    for parse_pair in parse_result {
        for iner in parse_pair.into_inner() {
            match iner.as_rule() {
                Rule::data => {}
                Rule::potential_delim => {
                    let next_delim = potential_delimiters.entry(iner.as_str()).or_insert(0);
                    *next_delim += 1;
                }
                _ => unreachable!(),
            };
        }
    }

    potential_delimiters
        .iter()
        .next_back()
        .unwrap()
        .0
        .to_owned()
        .to_owned()
}

fn expand_indices(input_indices: &[UnExpandedIndices], length: usize) -> (Vec<usize>, Vec<usize>) {
    // like moduluo  but number wraped  around index for negative numbers
    let tn = |num: i32| {
        if num >= 0 {
            num as usize
        } else {
            length - num as usize
        }
    };
    let expanded_indices: Vec<_> = input_indices
        .into_iter()
        .flat_map(|range| match range {
            UnExpandedIndices::Index(num) => vec![*num as usize],
            UnExpandedIndices::Range(BeginRange::FromStart, EndRange::ToEnd) => {
                (0..=length).collect()
            }
            UnExpandedIndices::Range(BeginRange::FromStart, EndRange::Index(num)) => {
                (0..=tn(*num)).collect()
            }
            UnExpandedIndices::Range(BeginRange::Index(num), EndRange::ToEnd) => {
                (tn(*num)..=length).collect()
            }
            UnExpandedIndices::Range(BeginRange::Index(begin_num), EndRange::Index(end_num)) => {
                (tn(*begin_num)..=tn(*end_num)).collect()
            }
        })
        .collect();

    let mut sorted_indices = expanded_indices.clone();
    sorted_indices.sort();

    (sorted_indices, expanded_indices)
}
