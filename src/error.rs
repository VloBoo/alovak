use std::fmt::{self, Display};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error{
    Io(std::io::Error),
    Other
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Io(error) => formatter.write_str(&error.to_string()),
            Error::Other =>  formatter.write_str("Other error"),
           // _ => !unimplemented!()
        }
    }
}

impl std::error::Error for Error {}
