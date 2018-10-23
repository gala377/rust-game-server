use std::collections::HashMap;

use super::errors::ReadError;
use super::{
    Request,
    RequestRaw,
    RequestId,
};

use byteorder::{ByteOrder, LittleEndian};


// Traits

/// Allows for usage of register! macro in factory_init
/// on request module name.
pub trait Registerable {
    fn id() -> RequestId;
    fn builder() -> BoxedReqBuilder;
}

// Types

// todo change Option to Result and write custom errors
// so we can send error response back.
/// Trait aliasing request builder function.
/// Builder function should create struct of type implementing Request
/// trait from raw data (being array of bytes).
/// None should be returned if reading was not possible.  
pub trait RequestBuilder: Fn(RequestRaw) -> Option<Box<dyn Request>> {}
impl<T> RequestBuilder for T where T: Fn(RequestRaw) -> Option<Box<dyn Request>> {}

/// Boxed RequestBuilder alias for shorter method signatures.
pub type BoxedReqBuilder = Box<dyn RequestBuilder<Output=Option<Box<dyn Request>>>>;


// Factory definition and declaration

/// Request factory returns Request trait object
/// from provided raw data.
pub struct RequestFactory {
    /// Maps request id to function creating it from raw bytes. 
    req_builders: HashMap<RequestId, BoxedReqBuilder>,
}

impl RequestFactory {

    /// Creates new RequestFactory
    pub fn new() -> RequestFactory {
        RequestFactory {
            req_builders: HashMap::new(),
        }
    }
    
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
    fn read_id(raw: &RequestRaw) -> RequestId {
        LittleEndian::read_u32(&raw[2..6])
    }

    // todo change to Result<(), RegisterError>
    /// Registers the builder function to the specified mess id.
    /// Doesn't allow to overwrite already existing builder.
    /// 
    /// Returns true if method was registered, false otherwise.
    /// Note that the only case that the false is returned is when
    /// there already exists builder on specified id.
    pub fn register(&mut self, id: RequestId, builder: BoxedReqBuilder) -> bool {
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
    #[allow(dead_code)]
    pub fn overregister(&mut self, id: RequestId, builder: BoxedReqBuilder) -> bool {
        if self.req_builders.contains_key(&id) {
            self.req_builders.insert(id, builder);
            true
        } else {
            false
        }
    }

    /// Registers builder if none exists. Overregisters otherwise.
    #[allow(dead_code)]
    pub fn force_register(&mut self, id: RequestId, builder: BoxedReqBuilder) {
        if self.req_builders.contains_key(&id) {
            self.overregister(id, builder);
        } else {
            self.register(id, builder);
        } 
    }
}