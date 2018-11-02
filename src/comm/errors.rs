use std::error::Error;
use std::convert::From;
use std::fmt;

use fast_from_derive::BadRequest;

/// General 400 status errors and some more (like connection severed).
#[derive(Debug)]
pub struct BadRequestError(Box<dyn Error>);

impl fmt::Display for BadRequestError {
    fn fmt(&self, f:  &mut fmt::Formatter) -> fmt::Result {
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
pub struct InternalServerError(Box<dyn Error>);

/// Returned if received request
/// had invalid headers server key. 
#[derive(Debug, BadRequest)]
pub struct HeaderValidationError {
    pub expected: Vec<u8>,
    pub actual: Vec<u8>,
}

impl Error for HeaderValidationError {
    fn cause(&self) -> Option<&dyn Error> {
        None
    }
}

impl fmt::Display for HeaderValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, 
            "Invalid message header: (expected {:?}, got {:?})",
            self.expected, self.actual
        )
    }
}

/// Returned when client unexpectedly closed the connection.
#[derive(Debug, BadRequest)]
pub struct ConnectionSevered;

impl fmt::Display for ConnectionSevered {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Client severed connection")
    }
}

impl Error for ConnectionSevered {
    fn cause(&self) -> Option<&dyn Error> {
        None
    }
}


#[derive(Debug, BadRequest)]
pub struct ReadError;

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Read error")
    }
}

impl Error for ReadError {
    fn cause(&self) -> Option<&dyn Error> {
        None
    }
}
