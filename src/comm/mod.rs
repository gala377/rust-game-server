// All of this module is considered WIP.

use std::any::Any;
use std::error::Error;
use std::io::Read;
use std::net::{TcpListener, TcpStream};

use super::config;

mod errors;
mod factory;
mod factory_init;
mod helpers;
mod requests;
mod request_dispatcher;
mod responses;

// Traits

/// General request interface representing
/// data that can be send to the game server.
pub trait Request {
    /// Returns identifier of the underlying request.
    fn id(&self) -> RequestId;

    /// Helper method needed to be implemented to allow for
    /// dynamic casting. Note that the default implementation is
    /// not possible (or maybe it would be but with a Boxed value).
    ///
    /// Because of that every struct implementing this should provide
    /// implementetion looking as such:
    ///     
    ///     fn as_any(&self) -> &dyn Any {
    ///         self
    ///     }
    ///
    /// Changing this implementation to the custom one needs to be
    /// supported with understandig of how dynamic dispatch and
    /// std::any::Any works.
    fn as_any(&self) -> &dyn Any;

    /// Helper method needed to be implemented to allow for
    /// dynamic casting. Note that the default implementation is
    /// not possible (or maybe it would be but with a Boxed value).
    ///
    /// Because of that every struct implementing this should provide
    /// implementetion looking as such:
    ///     
    ///     fn as_any(&mut self) -> &mut dyn Any {
    ///         self
    ///     }
    ///
    /// Changing this implementation to the custom one needs to be
    /// supported with understandig of how dynamic dispatch and
    /// std::any::Any works.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub trait Response {

    fn id(&self) -> ResponseId;
    fn payload(&self) -> Payload;
}

// Consts

/// Denotes max approved request and response lenght.
const MAX_MESS_LEN: usize = 512;
/// First 2 bytes identifying game request header.
const SKEY: &[u8; MSG_SKEY_FIELD_LEN] = b"RG";
const MSG_LEN_FIELD_LEN: usize = 4;
const MSG_SKEY_FIELD_LEN: usize = 2;
const MSG_HEADER_LEN: usize = MAX_MESS_LEN - (MSG_SKEY_FIELD_LEN + MSG_LEN_FIELD_LEN);
const MSG_PAYLOAD_LEN: usize = MAX_MESS_LEN - MSG_HEADER_LEN;

// Types

/// Raw data for the request factory to
/// crete the request from.
pub type RequestRaw = [u8; MAX_MESS_LEN];

/// Server requests identifier type.
pub type RequestId = u32;
/// Server response identifier type.
pub type ResponseId = u32;


pub type Payload = Vec<u8>;

// todo delete all the dead codes to know what is unneeded

/// Handles incoming connections and dispatches them
/// to Worker threads.
pub struct Server {
    #[allow(dead_code)]
    config: config::ServerConfig,
    listener: TcpListener,
    req_factory: factory::RequestFactory,
}

impl Server {
    /// Creates new server instance.
    /// Opens file with from the provided path. Then
    /// reads server documentation from it.
    pub fn new(filename: String) -> Server {
        println!("Creating server from file {}.", filename);
        let config = config::ServerConfig::from_file(filename.as_str()).unwrap();
        let listener = TcpListener::bind(config.to_string()).unwrap();
        println!("Created.");
        Server {
            config,
            listener,
            req_factory: factory_init::init(),
        }
    }

    /// Run waits for incoming connections. Then handles them taking requests
    /// and sending responses.  
    pub fn run(&self) {
        println!("Staring server.");
        for stream in self.listener.incoming() {
            let stream = stream.unwrap();
            println!("New connection established.");
            self.handle_connection(stream);
        }
    }

    // todo response? Thread pool?
    /// Reads message to a buffer. Returns on error.
    /// If no errors were present tries to interpret the message.
    fn handle_connection(&self, mut stream: TcpStream) {
        let raw = match self.read_mess(&mut stream) {
            Ok(buffer) => buffer,
            Err(val) => {
                println!("Error while handling connection: {}.\nAborting...", val);
                return;
            }
        };
        match self.parse_request(raw) {
            Err(err) => println!("Error while handling request {:?}", err),
            Ok(req) => {
                
            }
        };
    }

    // todo
    // Custom Error type
    /// Reads message from TcpStream to buffer. Then returns it.
    fn read_mess(&self, stream: &mut TcpStream) -> Result<RequestRaw, Box<Error>> {
        let mut buffer: RequestRaw = [0; MAX_MESS_LEN];
        match stream.read(&mut buffer) {
            Ok(n) => {
                println!("Read {} bytes. Proceeding.", n);
            }
            Err(err) => return Err(Box::new(err)),
        }
        Ok(buffer)
    }

    // todo <- implement
    fn parse_request(&self, raw: RequestRaw) -> Result<Box<dyn Request>, errors::ReadError> {
        if let Err(val) = self.validate_header(&raw[..]) {
            return Err(val);
        }
        self.req_factory.from_raw(raw)
    }

    fn validate_header(&self, raw: &[u8]) -> Result<(), errors::ReadError> {
        for i in 0..MSG_SKEY_FIELD_LEN {
            if SKEY[i] != raw[i] {
                return Err(errors::ReadError{});
            }
        }
        Ok(())
    }

    // todo implement 
    // fn handle_request(req: Box<dyn Request>) -> Result<Box<dyn Response>, errors::ExecutionError> {

    // }  

}
