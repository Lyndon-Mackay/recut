extern crate pest;

use io::{BufRead, BufReader, Read};
use std::{collections::HashMap, error, fmt, fs, io, num::ParseIntError};

#[macro_use]
extern crate pest_derive;

use fs::File;
use pest::{iterators::Pairs, Parser};

#[derive(Parser)]
#[grammar = "list.pest"]
pub struct ListParser;

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

enum BeginRange {
    Index(i32),
    FromStart,
}

enum EndRange {
    Index(i32),
    ToEnd,
}

enum UnExpandedIndices {
    Index(i32),
    Range(BeginRange, EndRange),
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
        CutType::FieldsRegexDelimiter(_, _) => {}
        CutType::FieldsStringDelimiter(_, _) => {}
    }

    Ok(())
}

/**
Attempts to pass the list argument if sucessfull it then converts the indices to numbers
and creates and generates a vecor of type  UnExpandedIndices
*/

/* TODO add range errors */
fn parse_indices(
    input: &str,
) -> std::result::Result<Vec<UnExpandedIndices>, pest::error::Error<Rule>> {
    /* Mao to act on sucessfull case transfroms parse into  */
    ListParser::parse(Rule::list, input).map(|parse| {
        parse
            .into_iter()
            .map(|parse_pair| {
                let range: Vec<_> = parse_pair.into_inner().map(|x| x.as_str()).collect();

                match range.as_slice() {
                    [index] => UnExpandedIndices::Index(index.parse().unwrap()),
                    [begin, end] if begin == &"" && end == &"" => {
                        UnExpandedIndices::Range(BeginRange::FromStart, EndRange::ToEnd)
                    }
                    [begin, end] if begin == &"" => UnExpandedIndices::Range(
                        BeginRange::FromStart,
                        EndRange::Index(end.parse().unwrap()),
                    ),
                    [begin, end] if end == &"" => UnExpandedIndices::Range(
                        BeginRange::Index(begin.parse().unwrap()),
                        EndRange::ToEnd,
                    ),
                    [begin, end] => UnExpandedIndices::Range(
                        BeginRange::Index(begin.parse().unwrap()),
                        EndRange::Index(end.parse().unwrap()),
                    ),
                    _ => unreachable!(),
                }
            })
            .collect()
    })
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

    let char_map = input_line
        .char_indices()
        .skip(first_index)
        .take(last_index)
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

    let byte_map = input_line
        .bytes()
        .enumerate()
        .skip(first_index)
        .take(last_index)
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
