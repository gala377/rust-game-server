use std::convert::From;
use std::error::Error;
use std::fmt;

use fast_from_derive::{BadRequest, SimpleError};

/// General 400 status errors and some more (like connection severed).
#[derive(Debug)]
pub struct BadRequestError(pub Box<dyn Error>);

impl fmt::Display for BadRequestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bad request: {}", self.0)
    }
}

impl Error for BadRequestError {
    fn cause(&self) -> Option<&dyn Error> {
        Some(self.0.as_ref())
    }
}

/// General 500 status errors.
#[derive(Debug)]
pub struct InternalServerError(pub Box<dyn Error>);

impl fmt::Display for InternalServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Internal server error: {}", self.0)
    }
}

impl Error for InternalServerError {
    fn cause(&self) -> Option<&dyn Error> {
        Some(self.0.as_ref())
    }
}

/// Returned if received request
/// had invalid headers server key.
#[derive(Debug, BadRequest, SimpleError)]
pub struct HeaderValidationError {
    pub expected: Vec<u8>,
    pub actual: Vec<u8>,
}

impl fmt::Display for HeaderValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Invalid message header: (expected {:?}, got {:?})",
            self.expected, self.actual
        )
    }
}

/// Returned when client unexpectedly closed the connection.
#[derive(Debug, BadRequest, SimpleError)]
pub struct ConnectionSevered;

impl fmt::Display for ConnectionSevered {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Client severed connection")
    }
}

#[derive(Debug, BadRequest, SimpleError)]
pub struct ReadError {
    couse: String,
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Read error {}", self.couse)
    }
}

impl From<String> for ReadError {
    fn from(couse: String) -> Self {
        ReadError { couse }
    }
}
