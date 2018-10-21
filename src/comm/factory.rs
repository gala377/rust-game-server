
use std::collections::HashMap;
use std::any::Any;

use super::errors::ReadError;

use byteorder::{ByteOrder, LittleEndian};

// Traits

/// Denotes general request
/// interface which can be send to the game server.
pub trait Request {

    fn id(&self) -> u32;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

// Type aliases for context

/// Raw data for the request factory to
/// crete the request from.
pub type RequestRaw = [u8; 512];

// Aliases builder function
pub trait RequestBuilder: Fn(RequestRaw) -> Option<Box<Request>> {}
impl<T> RequestBuilder for T where T: Fn(RequestRaw) -> Option<Box<Request>> {}


/// Boxed RequestBuilder alias for shorter method signatures

pub type BoxedReqBuilder = Box<dyn RequestBuilder<Output=Option<Box<Request>>>>;
// Factory definition and declaration

/// Request factory returns Request object
/// from provided raw data.
pub struct RequestFactory {
    req_builders: HashMap<u32, BoxedReqBuilder>,
}

impl RequestFactory {

    /// Creates new RequestFactory
    pub fn new() -> RequestFactory {
        RequestFactory {
            req_builders: HashMap::new(),
        }
    }

    // todo change to Result
    // todo implement
    /// Creates Request trait object from raw bytes data.
    /// Error if data was illformed or the builder function was not specified
    /// for the given id. // todo make it separate errors
    pub fn from_raw(&self, raw: RequestRaw) -> Result<Box<Request>, ReadError> {
        let id = Self::read_id(&raw);
        match self.req_builders.get(&id) {
            None => Err(ReadError{}),
            Some(builder) => {
                return match builder(raw) {
                    None => Err(ReadError),
                    Some(val) => Ok(val),
                }
            }
        }
    }

    /// Reads message id from the raw request.
    /// Note that the request needs to be valid in the first
    /// 6 bytes if it isn't this function panics.
    fn read_id(raw: &RequestRaw) -> u32 {
        LittleEndian::read_u32(&raw[2..6])
    }

    // todo change to Result<(), RegisterError>
    /// Registers the builder function to the specified mess id.
    /// Doesn't allow to overwrite already existing builder.
    /// 
    /// Returns true if method was registered, false otherwise.
    /// Note that the only case that the false is returned is when
    /// there already exists builder on specified id.
    pub fn register(&mut self, id: u32, builder: BoxedReqBuilder) -> bool {
        if self.req_builders.contains_key(&id) {
            false 
        } else {
            self.req_builders.insert(id, builder);
            true
        } 
    }

    // todo change to Result<(), OverrideError>
    /// Overrides existing builder. Returns true on success and false if 
    /// there is no builder already registered for the given id.
    pub fn overregister(&mut self, id: u32, builder: BoxedReqBuilder) -> bool {
        if self.req_builders.contains_key(&id) {
            self.req_builders.insert(id, builder);
            true
        } else {
            false
        }
    }

    /// Registers builder if none exists. Overregisters otherwise.
    pub fn force_register(&mut self, id: u32, builder: BoxedReqBuilder) {
        if self.req_builders.contains_key(&id) {
            self.overregister(id, builder);
        } else {
            self.register(id, builder);
        } 
    }
}