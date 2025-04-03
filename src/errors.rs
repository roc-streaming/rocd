use std::{error, fmt};

#[derive(Debug)]
pub struct ParseError {
    text: String,
}

impl ParseError {
    pub fn new<S: Into<String>>(text: S) -> Self {
        ParseError { text: text.into() }
    }
}

impl error::Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}
