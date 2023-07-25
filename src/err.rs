/*!
Error type to unify some error handling.
*/
use std::{
    error::Error,
    fmt::{Display, Formatter},
    io,
};

#[derive(Debug)]
pub enum FrErr {
    Io(io::Error),
    Regex(regex::Error),
}

impl From<io::Error> for FrErr {
    fn from(e: io::Error) -> Self {
        FrErr::Io(e)
    }
}

impl From<regex::Error> for FrErr {
    fn from(e: regex::Error) -> Self {
        FrErr::Regex(e)
    }
}

impl Display for FrErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FrErr::Regex(ref e) => write!(f, "regex error: {}", e),
            FrErr::Io(ref e) => write!(f, "I/O error: {}", &e),
        }
    }
}

impl Error for FrErr {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            FrErr::Io(ref e) => Some(e),
            FrErr::Regex(ref e) => Some(e),
        }
    }
}
