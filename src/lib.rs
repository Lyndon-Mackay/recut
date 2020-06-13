extern crate pest;

use io::{BufRead, BufReader, Read};
use std::{collections::HashMap, error, fmt, fs, io, num::ParseIntError};

#[macro_use]
extern crate pest_derive;

use fs::File;
use pest::{iterators::Pairs, Parser};
use range::{parse_indices, BeginRange, EndRange, UnExpandedIndices};
use regex::Regex;

mod range;

#[derive(Clone, Debug)]
pub enum IoType {
    FromStdIn,
    FromFile(String),
}

#[derive(Clone, Debug)]
pub enum CutType {
    Bytes(String, bool),
    Characters(String),
    FieldsInferDelimiter(String),
    FieldsRegexDelimiter(String, String),
    FieldsStringDelimiter(String, String),
}

#[derive(Clone, Debug)]
enum RangeParseError {
    IntError(ParseIntError),
}

impl CutType {
    fn ranges(&self) -> &str {
        match self {
            CutType::Bytes(s, _) => s,
            CutType::Characters(s) => s,
            CutType::FieldsInferDelimiter(s) => s,
            CutType::FieldsRegexDelimiter(s, _) => s,
            CutType::FieldsStringDelimiter(s, _) => s,
        }
    }
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
    let parsed_indices = parse_indices(cut_type.ranges()).unwrap();

    match cut_type {
        CutType::Bytes(_, split) => print_by_bytes(input, split, parsed_indices),
        CutType::Characters(_) => print_by_character(input, parsed_indices),
        CutType::FieldsInferDelimiter(_) => {}
        CutType::FieldsRegexDelimiter(_, delimiter) => {
            print_by_regex(input, &delimiter, parsed_indices)
        }
        CutType::FieldsStringDelimiter(_, delimiter) => {
            print_by_string_delimiter(input, &delimiter, parsed_indices)
        }
    }

    Ok(())
}

fn print_by_character(io_type: IoType, input_indices: Vec<UnExpandedIndices>) {
    match io_type {
        IoType::FromStdIn => {
            for line in io::stdin().lock().lines() {
                print_line_by_character(&line.unwrap(), &input_indices)
            }
        }
        IoType::FromFile(file_name) => {
            let file = File::open(file_name).unwrap();

            let reader = BufReader::new(file);

            for line in reader.lines() {
                print_line_by_character(&line.unwrap(), &input_indices);
            }
        }
    }
}

fn print_line_by_character(input_line: &str, input_indices: &[UnExpandedIndices]) {
    let length = input_line.chars().count();

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

fn print_by_bytes(io_type: IoType, splits_allowed: bool, input_indices: Vec<UnExpandedIndices>) {
    match io_type {
        IoType::FromStdIn => {
            for line in io::stdin().lock().lines() {
                print_line_by_bytes(&line.unwrap(), splits_allowed, &input_indices)
            }
        }
        IoType::FromFile(file_name) => {
            let file = File::open(file_name).unwrap();

            let reader = BufReader::new(file);

            for line in reader.lines() {
                print_line_by_bytes(&line.unwrap(), splits_allowed, &input_indices)
            }
        }
    }
}

fn print_line_by_bytes(input_line: &str, splits_alowed: bool, input_indices: &[UnExpandedIndices]) {
    let length = input_line.bytes().count();

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
        println!("{}", print_string.replace("�", ""))
    }
}

fn print_by_string_delimiter(
    io_type: IoType,
    delimiter: &str,
    input_indices: Vec<UnExpandedIndices>,
) {
    match io_type {
        IoType::FromStdIn => {
            for line in io::stdin().lock().lines() {
                print_line_by_string_delimiter(&line.unwrap(), delimiter, &input_indices)
            }
        }
        IoType::FromFile(file_name) => {
            let file = File::open(file_name).unwrap();

            let reader = BufReader::new(file);

            for line in reader.lines() {
                print_line_by_string_delimiter(&line.unwrap(), delimiter, &input_indices);
            }
        }
    }
}

fn print_line_by_string_delimiter(
    input_line: &str,
    delimiter: &str,
    input_indices: &[UnExpandedIndices],
) {
    let split_line = input_line.split(delimiter);

    let length = split_line.clone().count();

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

    let first_index = *sorted_indices.first().unwrap();

    let last_index = *sorted_indices.last().unwrap();

    let take_length = last_index + 1;

    let split_map = split_line
        .into_iter()
        .enumerate()
        .skip(first_index)
        .take(take_length)
        .collect::<HashMap<_, _>>();

    let mut print_string = Vec::with_capacity(input_line.len());

    for print_index in expanded_indices {
        print_string.push(*split_map.get(&print_index).unwrap())
    }

    println!("{}", print_string.join(delimiter));
}

fn print_by_regex(io_type: IoType, delimiter: &str, input_indices: Vec<UnExpandedIndices>) {
    let regex_delim = Regex::new(delimiter).unwrap();

    match io_type {
        IoType::FromStdIn => {
            for line in io::stdin().lock().lines() {
                print_line_by_regex_delimiter(&line.unwrap(), &regex_delim, &input_indices)
            }
        }
        IoType::FromFile(file_name) => {
            let file = File::open(file_name).unwrap();

            let reader = BufReader::new(file);

            for line in reader.lines() {
                print_line_by_regex_delimiter(&line.unwrap(), &regex_delim, &input_indices)
            }
        }
    }
}
fn print_line_by_regex_delimiter(
    input_line: &str,
    regex_delim: &Regex,
    input_indices: &[UnExpandedIndices],
) {
    let split_line: Vec<_> = regex_delim.split(input_line).collect();

    let length = split_line.len();

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

    let first_index = *sorted_indices.first().unwrap();

    let last_index = *sorted_indices.last().unwrap();

    let take_length = last_index + 1;

    let split_map = split_line
        .into_iter()
        .enumerate()
        .skip(first_index)
        .take(take_length)
        .collect::<HashMap<_, _>>();

    let mut print_string = Vec::with_capacity(input_line.len());

    for print_index in expanded_indices {
        print_string.push(*split_map.get(&print_index).unwrap())
    }

    println!("{}", print_string.join(""));
}

fn print_infer_regex(io_type: IoType, input_indices: Vec<UnExpandedIndices>) {}
