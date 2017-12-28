
use sxd_xpath;
use std::error;
use std::fmt;

use std::convert::From;

#[derive(Debug)]
pub enum TmxError {
    Xml(sxd_xpath::Error),
    // the TMX is intended to be a temporary setup until we get our own GUI,
    // so the errors in TMX parsing are not rigorous.
    Structure(String),
}

impl fmt::Display for TmxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            TmxError::Xml(ref xe) => write!(f, "error processing xml: {}", xe),
            TmxError::Structure(ref se) => write!(f, "{}", se),
        }
    }
}

impl error::Error for TmxError {
    fn description(&self) -> &str {
        "failure loading level from .tmx file"
    }
    fn cause(&self) -> Option<&error::Error> {
        match *self {
            TmxError::Xml(ref e) => Some(e),
            _ => None,
        }
    }
}

impl From<sxd_xpath::Error> for TmxError {
    fn from(e: sxd_xpath::Error) -> TmxError {
        TmxError::Xml(e)
    }
}

impl From<sxd_xpath::ParserError> for TmxError {
    fn from(e: sxd_xpath::ParserError) -> TmxError {
        TmxError::Xml(e.into())
    }
}

impl From<sxd_xpath::ExecutionError> for TmxError {
    fn from(e: sxd_xpath::ExecutionError) -> TmxError {
        TmxError::Xml(e.into())
    }
}

impl<'a> From<&'a str> for TmxError {
    fn from(s: &'a str) -> TmxError {
        TmxError::Structure(s.into())
    }
}

impl From<String> for TmxError {
    fn from(s: String) -> TmxError {
        TmxError::Structure(s)
    }
}
