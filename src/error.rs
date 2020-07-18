use crate::range;
use fmt::Display;
use pest::error::Error as PestError;
use std::{error, fmt, io, num::ParseIntError};

#[derive(Debug)]
pub enum RecutError {
    IntError(ParseIntError),
    InputError(io::Error),
    InputRangeParseError(PestError<range::Rule>),
    RegexError(regex::Error),
    RangeValueError(RangeError),
}

impl fmt::Display for RecutError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            //RangeParseError::RangeError(i, j) => write!(f, "{} should be less then {}", i, j),
            // This is a wrapper, so defer to the underlying types' implementation of `fmt`.
            RecutError::IntError(ref e) => e.fmt(f),
            RecutError::InputError(ref e) => e.fmt(f),
            RecutError::InputRangeParseError(ref e) => e.fmt(f),
            RecutError::RegexError(ref e) => e.fmt(f),
            RecutError::RangeValueError(ref e) => e.fmt(f),
        }
    }
}

impl error::Error for RecutError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            RecutError::IntError(ref e) => Some(e),
            RecutError::InputError(ref e) => Some(e),
            RecutError::InputRangeParseError(ref e) => Some(e),
            RecutError::RegexError(ref e) => Some(e),
            RecutError::RangeValueError(ref e) => Some(e),
        }
    }
}
impl From<ParseIntError> for RecutError {
    fn from(err: ParseIntError) -> RecutError {
        RecutError::IntError(err)
    }
}
impl From<io::Error> for RecutError {
    fn from(err: io::Error) -> RecutError {
        RecutError::InputError(err)
    }
}
impl From<PestError<range::Rule>> for RecutError {
    fn from(err: PestError<range::Rule>) -> Self {
        RecutError::InputRangeParseError(err)
    }
}
impl From<regex::Error> for RecutError {
    fn from(err: regex::Error) -> Self {
        RecutError::RegexError(err)
    }
}
#[derive(Debug)]
pub struct RangeError {}

impl Display for RangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "First number in range must be less then second number")
    }
}
impl error::Error for RangeError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}
