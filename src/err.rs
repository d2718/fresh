/*!
Error type to unify some error handling.
*/
use std::{
    error::Error,
    fmt::{Display, Formatter},
    io,
    ops::Deref,
};

use regex_chunker::RcErr;

#[derive(Debug)]
pub enum FrErr {
    Io(io::Error),
    Regex(regex::Error),
    Misc(Box<dyn Error>),
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

impl From<RcErr> for FrErr {
    fn from(e: RcErr) -> Self {
        match e {
            RcErr::Regex(e) => FrErr::Regex(e),
            RcErr::Read(e) => FrErr::Io(e),
            RcErr::Utf8(e) => FrErr::Misc(Box::new(e)),
        }
    }
}

impl Display for FrErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FrErr::Regex(ref e) => write!(f, "regex error: {}", e),
            FrErr::Io(ref e) => write!(f, "I/O error: {}", &e),
            FrErr::Misc(ref e) => write!(f, "{}", &e),
        }
    }
}

impl Error for FrErr {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            FrErr::Io(ref e) => Some(e),
            FrErr::Regex(ref e) => Some(e),
            FrErr::Misc(ref e) => Some(e.deref()),
        }
    }
}
