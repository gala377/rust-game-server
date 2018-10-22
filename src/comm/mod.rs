// All of this module is considered WIP.

use std::net::{TcpListener, TcpStream};
use std::io::Read;
use std::error::Error;
use std::any::Any;

use super::config;

mod factory;
mod requests;
mod errors;
mod helpers;


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


// Consts

/// Denotes max approved request and response lenght.
const MAX_MESS_LEN: usize = 512;
/// First 2 bytes identifying game request header.
#[allow(dead_code)]
const SKEY: &str = "RG";


// Types

/// Raw data for the request factory to
/// crete the request from.
pub type RequestRaw = [u8; MAX_MESS_LEN];

/// Server requests identifier type.
pub type RequestId = u32;


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
        Server{
            config,
            listener,
            req_factory: Server::init_req_factory(),
        }
    }

    // todo <- better method of registering requests.
    // maybe request having to implement
    // fn builder_fn() -> factory::BoxedReqBuilder ?
    /// Creates new request factory.
    /// Registers request builder functions.
    fn init_req_factory() -> factory::RequestFactory {
        let mut f = factory::RequestFactory::new();
        
        // todo is it possible (macro maybe?) for requests
        // to register themselves.
        if f.register(0, Box::new(|_raw: RequestRaw| {
            Some(Box::new(requests::hello::Req{}))
        })) {
            f
        } else {
            panic!("Could not register hello builder function");
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

    // todo response?
    /// Reads message to a buffer. Returns on error.
    /// If no errors were present tries to interpret the message.
    fn handle_connection(&self, mut stream: TcpStream) {
        let raw = match self.read_mess(&mut stream) {
            Ok(buffer) => buffer,
            Err(val) => {
                println!(
                    "Error while handling connection: {}.\nAborting...",
                    val);
                return;
            }
        };
        self.handle_request(raw);  
    }

    // todo
    // Custom Error type
    /// Reads message from TcpStream to buffer. Then returns it.
    fn read_mess(&self, stream: &mut TcpStream) -> Result<RequestRaw, Box<Error>> {
        let mut buffer: RequestRaw = [0; MAX_MESS_LEN];
        match stream.read(&mut buffer) {
            Ok(n) => {
                println!("Read {} bytes. Proceeding.", n);
            },
            Err(err) => return Err(Box::new(err)),
        }
        Ok(buffer)
    }

    // todo <- implement
    fn handle_request(&self, raw: RequestRaw) {
        match self.req_factory.from_raw(raw) {
            Ok(val) => {
                println!("It works, hello there!");
                let obj: &requests::hello::Req = helpers::cast::as_ref(&val).unwrap();
                println!("It works even more!");
                println!("My id is {}", obj.id());
            },
            Err(_) => println!("Unrecognized message"),
        }
    }
}