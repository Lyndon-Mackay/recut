use crate::error;
use error::{RangeError, RecutError};
use pest::Parser;
pub enum BeginRange {
    Index(i32),
    FromStart,
}

pub enum EndRange {
    Index(i32),
    ToEnd,
}

pub enum UnExpandedIndices {
    Index(i32),
    Range(BeginRange, EndRange),
}

#[derive(Parser)]
#[grammar = "list.pest"]
pub struct ListParser;

/**
Attempts to pass the list argument if sucessfull it then converts the indices to numbers
and creates and generates a vecor of type  UnExpandedIndices
*/

/* TODO add range errors */
pub fn parse_indices(input: &str) -> std::result::Result<Vec<UnExpandedIndices>, RecutError> {
    /* Map to act on sucessfull case transfroms parse into  */
    let parse = ListParser::parse(Rule::list, input)?;
    parse
        .into_iter()
        .map(|parse_pair| {
            let range: Vec<_> = parse_pair.into_inner().map(|x| x.as_str()).collect();

            match range.as_slice() {
                [index] => Ok(UnExpandedIndices::Index(index.parse()?)),
                [begin, end] if begin == &"" && end == &"" => Ok(UnExpandedIndices::Range(
                    BeginRange::FromStart,
                    EndRange::ToEnd,
                )),
                [begin, end] if begin == &"" => Ok(UnExpandedIndices::Range(
                    BeginRange::FromStart,
                    EndRange::Index(end.parse()?),
                )),
                [begin, end] if end == &"" => Ok(UnExpandedIndices::Range(
                    BeginRange::Index(begin.parse()?),
                    EndRange::ToEnd,
                )),
                [begin, end] => check_range(begin.parse()?, end.parse()?),
                _ => unreachable!(),
            }
        })
        .collect::<Result<Vec<UnExpandedIndices>, RecutError>>()
}

fn check_range(first_num: i32, second_num: i32) -> Result<UnExpandedIndices, RecutError> {
    if first_num < second_num {
        Ok(UnExpandedIndices::Range(
            BeginRange::Index(first_num),
            EndRange::Index(second_num),
        ))
    } else {
        Err(RecutError::RangeValueError(RangeError {}))
    }
}
