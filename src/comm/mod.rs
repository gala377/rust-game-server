// All of this module is considered WIP

use std::io::{
    Read,
    Write,
};
use std::net::{
    TcpListener,
    TcpStream,
};
use std::thread;
use std::sync::{
    Arc,
    RwLock,
};

use byteorder::{
    LittleEndian,
    ByteOrder,
};

use super::config;

mod errors;
mod handlers;


pub type MessageRaw = Vec<u8>;
pub type MessageId = u32;
pub type Payload = Vec<u8>;


pub trait Message {
    fn id(&self) -> MessageId;
    fn payload(&self) -> Payload;
}

pub trait Request: Message {}
impl<T> Request for T where T: Message {}

pub trait Response: Message {}
impl<T> Response for T where T: Message {}


const SKEY: &[u8; MSG_SKEY_FIELD_LEN] = b"RG";
const MSG_BATCH_LEN: usize = 512;
const MSG_LEN_FIELD_LEN: usize = 4;
const MSG_SKEY_FIELD_LEN: usize = 2;
const MSG_ID_FIELD_LEN: usize = 4;
const MSG_HEADER_LEN: usize = MSG_SKEY_FIELD_LEN + MSG_LEN_FIELD_LEN + MSG_ID_FIELD_LEN;


// todo delete all the dead codes to know what is unneeded

/// Handles incoming connections and dispatches them
/// to Worker threads.
pub struct Server {
    #[allow(dead_code)]
    config: config::ServerConfig,
    listener: TcpListener,
    thread_handles: Vec<thread::JoinHandle<()>>,
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

            thread_handles: Vec::new(),
        }
    }

    /// Run waits for incoming connections. Then handles them taking requests
    /// and sending responses.  
    pub fn run(&mut self) {
        println!("Server: Staring listening.");
        let local_hand_arc = Arc::new(RwLock::new(
            handlers::init::new_dispatcher()
        ));
        for stream in self.listener.incoming() {
            let stream = stream.unwrap();
            println!("Server: New connection established.");
            let conn_handler = ConnectionHandler::new(local_hand_arc.clone());
            self.thread_handles.push(
                thread::spawn(move || {
                    println!("HandlerThread: New thread handling connection!");
                    conn_handler.handle_connection(stream);
                    println!("HandlerThread: Connection handled!");
                })
            );
        }
        for handle in self.thread_handles.drain(..) {
            if let Err(_) = handle.join() {
                println!("Error while joining a thread!");
            }
        }
    }




}

struct ConnectionHandler {
    req_handlers: Arc<RwLock<handlers::Dispatcher>>,
}

impl ConnectionHandler {

    fn new(req_handlers: Arc<RwLock<handlers::Dispatcher>>) -> ConnectionHandler {
        ConnectionHandler {
            req_handlers,
        }
    }

        // todo response? Thread pool?
    // note that now it olny handles single message
    // we need to make it an open communication.
    /// Reads message to a buffer. Returns on error.
    /// If no errors were present tries to interpret the message.
    fn handle_connection(&self, mut stream: TcpStream) {
        println!("Server: Trying to build message!");
        let raw = match Self::read_mess(&mut stream) {
            Ok(buffer) => buffer,
            Err(val) => {
                println!("Server: Error while building message.\nAborting...");
                return;
            }
        };
        println!("Server: Message assembled. Request parsing!");
        
        match self.req_handlers.read() {
            Ok(guard) => {
                match (*guard).dispatch_from_raw(raw) {
                    Err(err) => println!("Server: Error while handling request {:?}", err),
                    Ok(resp) => {
                        println!("Server: Got response!");
                        match stream.write_all(&Self::response_as_bytes(resp)[..]) {
                            Ok(_) => println!("Message sent successfully!"),
                            Err(err) => println!("Error while sending the response {}", err),
                        }
                        stream.flush().unwrap();
                    }
                };
            },
            Err(err) => println!("Error while getting a lock!"),
        }
    }

    /// todo
    // Custom Error type
    /// Reads raw message from TcpStream to buffer. Then returns it.
    fn read_mess(stream: &mut TcpStream) -> Result<MessageRaw, errors::ReadError> {
        let mut buffer = [0; MSG_BATCH_LEN];
        let mut raw = Vec::with_capacity(MSG_HEADER_LEN); 
        
        let mut header_parsed = false;
        let mut full_msg_len = 0;
        loop {
            match stream.read(&mut buffer) {
                Ok(n) => {
                    match n {
                        0 => {
                            println!("Server: Connection severed!");
                            return Err(errors::ReadError{});
                        },
                        _ => {
                            println!("Server: Read {} bytes. Proceeding.", n);
                            raw.extend_from_slice(&buffer[0..n]);
                        },
                    }
                }
                Err(err) => return Err(errors::ReadError{}),
            }
            if raw.len() >= MSG_HEADER_LEN && !header_parsed {
                println!("Server: Read sufficient number of bytes to parse header.");
                full_msg_len = Self::parse_header(&raw[..])?;
                full_msg_len += MSG_HEADER_LEN as u32;
                println!("Server: Full msg is {} bytes. {} more bytes to read", full_msg_len, full_msg_len as usize - raw.len());
                header_parsed = true;
            }
            if raw.len() == full_msg_len as usize && header_parsed {
                println!("Server: Read all the payload bytes.");
                break;
            } else if raw.len() > full_msg_len as usize && header_parsed {
                println!("Server: Read more than specified in payload len. Aborting...");
                return Err(errors::ReadError{});
            }
        }
        Ok(raw)
    }

    fn parse_header(header: &[u8]) -> Result<u32, errors::ReadError> {
        for i in 0..MSG_SKEY_FIELD_LEN {
            if SKEY[i] != header[i] {
                return Err(errors::ReadError{});
            }
        }
        let beggining = MSG_SKEY_FIELD_LEN + MSG_ID_FIELD_LEN;
        Ok(LittleEndian::read_u32(&header[beggining..beggining+MSG_ID_FIELD_LEN]))
    }   

    fn response_as_bytes(resp: Box<dyn Response>) -> Vec<u8> {
        let mut as_bytes = Vec::with_capacity(MSG_HEADER_LEN);
        for &ch in SKEY.iter() {
            as_bytes.push(ch);
        };
        let mut buff = [0; 4];
        LittleEndian::write_u32(&mut buff, resp.id());    
        as_bytes.extend_from_slice(&buff);
        
        let payload = resp.payload();
        LittleEndian::write_u32(&mut buff, payload.len() as u32);
        as_bytes.extend_from_slice(&buff);
        as_bytes.reserve(payload.len());
        as_bytes.extend_from_slice(&payload[..]);

        as_bytes
    } 

}