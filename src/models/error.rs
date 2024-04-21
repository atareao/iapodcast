use std::{
    fmt::{
        Display,
        Formatter,
        Result
    },
    str::Utf8Error,
    error::Error as StdError,
    io::Error as IoError,
    num::ParseIntError,
};
use serde::{Serialize, ser::SerializeStruct};


#[derive(Debug)]
pub struct Error{
    details: String,
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::prelude::v1::Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        let mut state = serializer.serialize_struct("Error", 1)?;
        state.serialize_field("details", &self.details)?;
        state.end()
    }
}

impl Error{
    pub fn new(msg: &str) -> Self{
        Error{
            details: msg.to_string(),
        }
    }
}

impl Display for Error{
    fn fmt(&self, f: &mut Formatter) -> Result{
        write!(f, "{}", self.details)
    }
}

impl From<minijinja::Error> for Error{
    fn from(error: minijinja::Error) -> Self{
        Error::new(&error.to_string())
    }
}


impl StdError for Error {
    fn description(&self) -> &str {
        &self.details
    }
}

impl From<IoError> for Error{
    fn from(error: IoError) -> Self{
        Error::new(&error.to_string())
    }
}

impl From<ParseIntError> for Error{
    fn from(error: ParseIntError) -> Self{
        Error::new(&error.to_string())
    }
}

impl From<Utf8Error> for Error{
    fn from(error: Utf8Error) -> Self{
        Error::new(&error.to_string())
    }
}

impl From<rss::Error> for Error{
    fn from(error: rss::Error) -> Self{
        Error::new(&error.to_string())
    }
}
