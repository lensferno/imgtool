use std::{error, fmt, io};
use std::num::{ParseFloatError, ParseIntError};
use std::str::ParseBoolError;

#[derive(Debug)]
pub struct ImageProcessError {
    msg: String,
}

impl ImageProcessError {
    pub fn new(msg: String) -> Self {
        Self { msg }
    }
}

impl error::Error for ImageProcessError {}

impl fmt::Display for ImageProcessError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "IOError: {}", self.msg)
    }
}

impl From<io::Error> for ImageProcessError {
    fn from(err: io::Error) -> Self {
        Self::new(format!("{}", err))
    }
}

impl From<caesium::error::CaesiumError> for ImageProcessError {
    fn from(err: caesium::error::CaesiumError) -> Self {
        Self::new(format!("{}", err))
    }
}

#[derive(Debug)]
pub struct ValueParseError {
    msg: String,
}

impl ValueParseError {
    pub fn new(msg: String) -> Self {
        Self { msg }
    }
}

impl From<&str> for ValueParseError {
    fn from(msg: &str) -> Self {
        Self {
            msg: msg.to_string(),
        }
    }
}

impl From<String> for ValueParseError {
    fn from(msg: String) -> Self {
        Self { msg }
    }
}

impl error::Error for ValueParseError {}

impl fmt::Display for ValueParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ValueParseErr: {}", self.msg)
    }
}

impl From<ParseIntError> for ValueParseError {
    fn from(err: ParseIntError) -> Self {
        Self::new(format!("ParseIntErr: {}", err))
    }
}

impl From<ParseBoolError> for ValueParseError {
    fn from(err: ParseBoolError) -> Self {
        Self::new(format!("ParseBoolErr: {}", err))
    }
}

impl From<ParseFloatError> for ValueParseError {
    fn from(err: ParseFloatError) -> Self {
        Self::new(format!("ParseFloatErr: {}", err))
    }
}
