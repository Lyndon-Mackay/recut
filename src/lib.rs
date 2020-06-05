extern crate pest;

use io::Read;
use std::{error, fmt, fs, io, num::ParseIntError};

#[macro_use]
extern crate pest_derive;

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
    Bytes(String),
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
    Index(String),
    Range(BeginRange, EndRange),
}

#[derive(Clone, Debug)]
enum RangeParseError {
    IntError(ParseIntError),
    RangeError(i8, i8),
}

impl fmt::Display for RangeParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RangeParseError::RangeError(i, j) => write!(f, "{} should be less then {}", i, j),
            // This is a wrapper, so defer to the underlying types' implementation of `fmt`.
            RangeParseError::IntError(ref e) => e.fmt(f),
        }
    }
}

impl error::Error for RangeParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            RangeParseError::IntError(ref e) => Some(e),
            RangeParseError::RangeError(_, _) => None,
        }
    }
}
impl From<ParseIntError> for RangeParseError {
    fn from(err: ParseIntError) -> RangeParseError {
        RangeParseError::IntError(err)
    }
}

pub fn cut(input: IoType) -> Result<(), io::ErrorKind> {
    let contents = match input {
        IoType::FromFile(filename) => match fs::read_to_string(filename) {
            Ok(s) => Ok(s),
            Err(e) => Err(e),
        },
        IoType::FromStdIn => {
            let mut io_string = String::new();
            match io::stdin().read_to_string(&mut io_string) {
                Ok(_) => Ok(io_string),
                Err(e) => match e.kind() {
                    io::ErrorKind::BrokenPipe => Ok(io_string),
                    _ => Err(e),
                },
            }
        }
    };

    parse_indices("-1:9,-19:-1,4:,:15,3,-2,9:");

    Ok(())
}

fn parse_indices(input: &str) -> Vec<UnExpandedIndices> {
    let parse = ListParser::parse(Rule::list, input).expect("unsuccessful parse");

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
}

/*
    Returns a list of indices or the first parsing or range error encountered
*/
/*
fn generate_indices(input: &str) -> Result<Vec<i8>, RangeParseError> {
    let unexpanded_ranges = input.split(",");

    let unparsed_ranges: Vec<UnParsedPosition> = unexpanded_ranges
        .map(|s| {
            if s.contains(":") {
                let ranges: Vec<String> = s.splitn(2, ":").map(|v| v.to_string()).collect();
                UnParsedPosition::Range(ranges[0].clone(), ranges[1].clone())
            } else {
                UnParsedPosition::Index(s.to_string())
            }
        })
        .collect();

    let parsed_expanded_ranges: Vec<Result<Vec<i8>, RangeParseError>> = unparsed_ranges
        .into_iter()
        .map(|s| match s {
            UnParsedPosition::Index(x) => match x.parse::<i8>() {
                Ok(num) => Ok(vec![num]),
                Err(x) => Err(RangeParseError::IntError(x)),
            },
            UnParsedPosition::Range(x, y) => {
                let x = x.parse::<i8>();
                let y = y.parse::<i8>();

                match (x, y) {
                    (Ok(i), Ok(j)) => {
                        if i < j {
                            Ok((i..j).into_iter().collect())
                        } else {
                            Err(RangeParseError::RangeError(i, j))
                        }
                    }
                    (Ok(_), Err(err)) => Err(RangeParseError::IntError(err)),
                    (Err(err), Ok(_)) => Err(RangeParseError::IntError(err)),
                    (Err(err), Err(_)) => Err(RangeParseError::IntError(err)),
                }
            }
        })
        .collect();

    match parsed_expanded_ranges.iter().find(|x| x.is_err()) {
        Some(err) => Err(err.clone().unwrap_err()),
        None => Ok(parsed_expanded_ranges
            .into_iter()
            .filter_map(|x| match x {
                Ok(y) => Some(y),
                Err(_) => None,
            })
            .flat_map(|v| v)
            .collect::<Vec<i8>>()),
    }
}*/
