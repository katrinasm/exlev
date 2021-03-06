use std::error;
use std::fmt;

#[derive(Debug, Clone, Copy)]
pub enum DecodeError {}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "error decoding binary level")
    }
}

impl error::Error for DecodeError {
    fn description(&self) -> &str {
        "failure loading level from binary"
    }
    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EncodeError {}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "error encoding binary level")
    }
}

impl error::Error for EncodeError {
    fn description(&self) -> &str {
        "failure writing level to binary"
    }
    fn cause(&self) -> Option<&error::Error> {
        None
    }
}
