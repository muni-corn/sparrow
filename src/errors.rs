use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io;

pub type SparrowResult<T> = Result<T, SparrowError>;

#[derive(Debug)]
pub enum SparrowError {
    InputCanceled,
    BasicMessage(String),
    Io(io::Error),
    ChronoParse(chrono::ParseError),
    YamlError(serde_yaml::Error),
}

impl Display for SparrowError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InputCanceled => write!(f, "input canceled"),
            Self::BasicMessage(b) => write!(f, "sparrow hit an error: {}", b),
            Self::Io(i) => write!(f, "there was an i/o error: {}", i),
            Self::ChronoParse(e) => e.fmt(f),
            Self::YamlError(y) => y.fmt(f),
        }
    }
}

impl Error for SparrowError {}

impl From<chrono::ParseError> for SparrowError {
    fn from(e: chrono::ParseError) -> Self {
        Self::ChronoParse(e)
    }
}

impl From<io::Error> for SparrowError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<serde_yaml::Error> for SparrowError {
    fn from(e: serde_yaml::Error) -> Self {
        Self::YamlError(e)
    }
}
