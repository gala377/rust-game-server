use std::{
    convert::From,
    io::{Read, Write},
    iter::FromIterator,
    net::{TcpListener, TcpStream},
    sync::{Arc, RwLock},
    thread,
};

use byteorder::{ByteOrder, LittleEndian};

use super::config;

mod errors;
mod handlers;

/// Alias for vector of bytes.
/// Used to stress that the vector should contain
/// all of the request.
pub type MessageRaw = Vec<u8>;
/// Type by witch Messages can be identified.
pub type MessageId = u32;
/// Alias for vector of bytes.
/// Used to stress that the vector should contain
/// Message payload data.
pub type Payload = Vec<u8>;

/// Generic Message trait handled and returned
/// from the server instance.
pub trait Message {
    fn id(&self) -> MessageId;
    fn payload(&self) -> Payload;
}

/// Alias to Message.
/// Used to stress that the current Message
/// is treated as request to the server.
pub trait Request: Message {}
impl<T> Request for T where T: Message {}

/// Alias to Message.
/// Used to stress that the current Message
/// is treated as response from the server.
pub trait Response: Message {}
impl<T> Response for T where T: Message {}

const SKEY: &[u8; MSG_SKEY_FIELD_LEN] = b"RG";
const MSG_BATCH_LEN: usize = 512;
const MSG_LEN_FIELD_LEN: usize = 4;
const MSG_SKEY_FIELD_LEN: usize = 2;
const MSG_ID_FIELD_LEN: usize = 4;
const MSG_HEADER_LEN: usize = MSG_SKEY_FIELD_LEN + MSG_LEN_FIELD_LEN + MSG_ID_FIELD_LEN;

/// Handles incoming connections and dispatches them
/// to Worker threads.
pub struct Server {
    listener: TcpListener,
    req_handlers: Arc<RwLock<handlers::Dispatcher>>,
    thread_handles: Vec<thread::JoinHandle<()>>,
}

impl Server {
    /// Creates new server instance.
    /// Opens file with from the provided path.
    /// Then reads server configuration from it.
    pub fn new(filename: String) -> Server {
        println!("Creating server from file {}.", filename);
        let config = config::ServerConfig::from_file(filename.as_str()).unwrap();
        let listener = TcpListener::bind(config.to_string()).unwrap();
        println!("Created.");
        Server {
            listener,
            req_handlers: Arc::new(RwLock::new(handlers::init::new_dispatcher())),

            thread_handles: Vec::new(),
        }
    }

    /// Run waits for incoming connections.
    /// If one appears handles it in new thread.
    pub fn run(&mut self) {
        println!("Server: Staring listening.");
        let mut conn_count: usize = 0;
        for stream in self.listener.incoming() {
            let stream = stream.unwrap();
            println!("Server: New connection established.");
            let conn_handler = ConnectionHandler::new(conn_count, self.req_handlers.clone());
            self.thread_handles.push(thread::spawn(move || {
                println!("HandlerThread: New thread handling connection!");
                conn_handler.handle_connection(stream);
                println!("HandlerThread: Connection handled!");
            }));
            conn_count += 1;
        }
    }
}

impl Drop for Server {
    /// Joins on currently running connection
    /// handling threads.
    fn drop(&mut self) {
        for handle in self.thread_handles.drain(..) {
            if let Err(err) = handle.join() {
                println!("Error while joining a thread! {:?}", err);
            }
        }
    }
}

struct ConnectionHandler {
    id: usize,
    req_handlers: Arc<RwLock<handlers::Dispatcher>>,
}

impl ConnectionHandler {
    fn new(id: usize, req_handlers: Arc<RwLock<handlers::Dispatcher>>) -> ConnectionHandler {
        ConnectionHandler { id, req_handlers }
    }

    // has no meanings of stopping.
    // communication channel on which we can check to see if we should close?
    // 
    // whats more it needs to be stateful.
    // maybe &mut self and sending reference to connection 
    // to dispatch from raw?
    fn handle_connection(&self, mut stream: TcpStream) {
        loop {
            println!("ConnHandler[{}]: Trying to build message!", &self.id);
            let raw = match self.read_mess(&mut stream) {
                Ok(buffer) => buffer,
                Err(err) => {
                    println!(
                        "ConnHandler[{}]: Error while building message: {}.\nAborting...",
                        &self.id, err,
                    );
                    return;
                }
            };
            println!(
                "ConnHandler[{}]: Message assembled. Request parsing!",
                &self.id
            );

            match self.req_handlers.read() {
                Ok(guard) => {
                    match (*guard).dispatch_from_raw(raw) {
                        Err(err) => println!(
                            "ConnHandler[{}]: Error while handling request {:?}",
                            &self.id, err
                        ),
                        Ok(resp) => {
                            println!("ConnHandler[{}]: Got response!", &self.id);
                            match stream.write_all(&Self::response_as_bytes(resp)[..]) {
                                Ok(_) => {
                                    println!("ConnHandler[{}]: Message sent successfully!", &self.id)
                                }
                                Err(err) => println!(
                                    "ConnHandler[{}]: Error while sending the response {}",
                                    &self.id, err
                                ),
                            }
                            stream.flush().unwrap();
                        }
                    };
                }
                Err(err) => {
                    println!("ConnHandler[{}]: Error while getting a lock! {}", &self.id, err);
                    return;
                }
            }
        }
    }

    fn read_mess(&self, stream: &mut TcpStream) -> Result<MessageRaw, errors::BadRequestError> {
        let mut buffer = [0; MSG_BATCH_LEN];
        let mut raw = Vec::with_capacity(MSG_HEADER_LEN);

        let mut header_parsed = false;
        let mut full_msg_len = 0;
        loop {
            match stream.read(&mut buffer) {
                Ok(n) => match n {
                    0 => {
                        println!("ConnHandler[{}]: Connection severed!", &self.id);
                        return Err(errors::BadRequestError::from(errors::ConnectionSevered{}));
                    }
                    _ => {
                        println!("ConnHandler[{}]: Read {} bytes. Proceeding.", &self.id, n);
                        raw.extend_from_slice(&buffer[0..n]);
                    }
                },
                Err(err) => return Err(
                    errors::BadRequestError::from(
                        errors::ReadError::from(err.to_string()))),
            }
            if raw.len() >= MSG_HEADER_LEN && !header_parsed {
                println!(
                    "ConnHandler[{}]: Read sufficient number of bytes to parse header.",
                    &self.id
                );
                full_msg_len = Self::parse_header(&raw[..])?;
                full_msg_len += MSG_HEADER_LEN as u32;
                println!(
                    "ConnHandler[{}]: Full msg is {} bytes. {} more bytes to read",
                    &self.id,
                    full_msg_len,
                    full_msg_len as usize - raw.len()
                );
                header_parsed = true;
            }
            if raw.len() == full_msg_len as usize && header_parsed {
                println!("ConnHandler[{}]: Read all the payload bytes.", &self.id);
                break;
            } else if raw.len() > full_msg_len as usize && header_parsed {
                println!(
                    "ConnHandler[{}]: Read more than specified in payload len. Aborting...",
                    &self.id
                );
                return Err(
                    errors::BadRequestError::from(
                        errors::ReadError::from(format!(
                            "read more bytes than specified in mess len. 
                            Expected: {}, Read: {}",
                            full_msg_len,
                            raw.len(), 
                ))));
            }
        }
        Ok(raw)
    }

    fn parse_header(header: &[u8]) -> Result<u32, errors::HeaderValidationError> {
        for i in 0..MSG_SKEY_FIELD_LEN {
            if SKEY[i] != header[i] {
                return Err(errors::HeaderValidationError {
                    expected: Vec::from_iter(SKEY.iter().cloned()),
                    actual: Vec::from_iter(header.iter().cloned()),
                });
            }
        }
        let beggining = MSG_SKEY_FIELD_LEN + MSG_ID_FIELD_LEN;
        Ok(LittleEndian::read_u32(
            &header[beggining..beggining + MSG_ID_FIELD_LEN],
        ))
    }

    fn response_as_bytes(resp: Box<dyn Response>) -> Vec<u8> {
        let mut as_bytes = Vec::with_capacity(MSG_HEADER_LEN);
        for &ch in SKEY.iter() {
            as_bytes.push(ch);
        }
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
