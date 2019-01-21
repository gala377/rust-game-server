use std::{
    convert::From,
    io::{Read, Write},
    iter::FromIterator,
    net::{TcpListener, TcpStream},
    sync::{Arc, RwLock},
    thread,
};


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
    req_dispatcher: Arc<RwLock<handlers::Dispatcher>>,
    thread_handles: Vec<thread::JoinHandle<()>>,
}

impl Server {
    /// Creates new server instance.
    /// Opens file from the provided path.
    /// Then reads server configuration from it.
    pub fn new(filename: String) -> Server {
        eprintln!("[{:^15}]: Creating server from file {}.", "Initialization", filename);
        let config = config::ServerConfig::from_file(filename.as_str()).unwrap();
        let listener = TcpListener::bind(config.to_string()).unwrap();
        eprintln!("[{:^15}]: Created.", "Initialization");
        Server {
            listener,
            req_dispatcher: Arc::new(RwLock::new(handlers::init::new_dispatcher())),

            thread_handles: Vec::new(),
        }
    }

    /// Run waits for incoming connections.
    /// If one appears handles it in new thread.
    pub fn run(&mut self) {
        eprintln!("[{:^15}]: Staring listening.", "Server");
        let mut conn_count: usize = 0;
        for stream in self.listener.incoming() {
            let stream = stream.unwrap();
            eprintln!("[{:^15}]: New connection established.", "Server");
            let conn_handler = ConnectionHandler::new(conn_count, self.req_dispatcher.clone());
            self.thread_handles.push(thread::spawn(move || {
                eprintln!("[{:^15}]: New thread handling connection!", "HandlerThread");
                conn_handler.handle_connection(stream);
                eprintln!("[{:^15}]: Connection handled!", "HandlerThread");
            }));
            conn_count += 1;
        }
    }
}

// todo: should actually stop these threads immediatly
// not wait for them
impl Drop for Server {
    /// Joins on currently running connection
    /// handling threads.
    fn drop(&mut self) {
        for handle in self.thread_handles.drain(..) {
            if let Err(err) = handle.join() {
                eprintln!("[{:^15}]: Error while joining a thread! {:?}", "Server", err);
            }
        }
    }
}

struct ConnectionHandler {
    id: usize,
    req_handlers: Arc<RwLock<handlers::Dispatcher>>,
}

// todo: refactor
// read_mess and handle_connection are too looong
impl ConnectionHandler {
    fn new(id: usize, req_handlers: Arc<RwLock<handlers::Dispatcher>>) -> ConnectionHandler {
        ConnectionHandler { id, req_handlers }
    }

    // todo:
    // has no meanings of stopping.
    // communication channel on which we can check to see if we should close?
    //
    // whats more it needs to be stateful.
    // maybe &mut self and sending reference to connection
    // to dispatch from raw?
    // Or some kind of reference to a context struct being passed along? 
    fn handle_connection(&self, mut stream: TcpStream) {
        loop {
            eprintln!("[{:^12}[{}]]: Trying to build message!", "ConnHandler", &self.id);
            let raw = match self.read_mess(&mut stream) {
                Ok(buffer) => buffer,
                Err(err) => {
                    eprintln!(
                        "[{:^12}[{}]]: Error while building message: \"{}\". Aborting...",
                        "ConnHandler", &self.id, err,
                    );
                    return;
                }
            };
            eprintln!(
                "[{:^12}[{}]]: Message assembled. Request parsing!",
                "ConnHandler", &self.id
            );

            match self.req_handlers.read() {
                Ok(guard) => {
                    match (*guard).dispatch_from_raw(raw) {
                        // todo method to return error response from error
                        Err(err) => eprintln!(
                            "[{:^12}[{}]]: Error while handling request {:?}",
                            "ConnHandler", &self.id, err
                        ),
                        Ok(resp) => {
                            eprintln!("[{:^12}[{}]]: Got response!", "ConnHandler", &self.id);
                            match stream.write_all(&Self::response_as_bytes(resp)[..]) {
                                Ok(_) => eprintln!(
                                    "[{:^12}[{}]]: Message sent successfully!",
                                    "ConnHandler", &self.id
                                ),
                                Err(err) => eprintln!(
                                    "[{:^12}[{}]]: Error while sending the response \"{}\"",
                                    "ConnHandler", &self.id, err
                                ),
                            }
                            stream.flush().unwrap();
                        }
                    };
                }
                Err(err) => {
                    eprintln!(
                        "[{:^12}[{}]]: Error while getting a lock! {}",
                        "ConnHandler", &self.id, err
                    );
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
                        eprintln!("[{:^12}[{}]]: Connection severed!", "ConnHandler", &self.id);
                        return Err(errors::BadRequestError::from(errors::ConnectionSevered {}));
                    }
                    _ => {
                        eprintln!("[{:^12}[{}]]: Read {} bytes. Proceeding.", "ConnHandler", &self.id, n);
                        raw.extend_from_slice(&buffer[0..n]);
                    }
                },
                Err(err) => {
                    return Err(errors::BadRequestError::from(errors::ReadError::from(
                        err.to_string(),
                    )))
                }
            }
            if raw.len() >= MSG_HEADER_LEN && !header_parsed {
                eprintln!(
                    "[{:^12}[{}]]: Read sufficient number of bytes to parse header.",
                    "ConnHandler", &self.id
                );
                full_msg_len = Self::parse_header(&raw[..])?;
                full_msg_len += MSG_HEADER_LEN as u32;
                eprintln!(
                    "[{:^12}[{}]]: Full msg is {} bytes. {} more bytes to read",
                    "ConnHandler",
                    &self.id,
                    full_msg_len,
                    full_msg_len as usize - raw.len()
                );
                header_parsed = true;
            }
            if raw.len() == full_msg_len as usize && header_parsed {
                eprintln!("[{:^12}[{}]]: Read all the payload bytes.", "ConnHandler", &self.id);
                break;
            } else if raw.len() > full_msg_len as usize && header_parsed {
                eprintln!(
                    "[{:^12}[{}]]: Read more than specified in payload len. Aborting...",
                    "ConnHandler", &self.id
                );
                return Err(errors::BadRequestError::from(errors::ReadError::from(
                    format!(
                        "read more bytes than specified in mess len. 
                            Expected: {}, Read: {}",
                        full_msg_len,
                        raw.len(),
                    ),
                )));
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
        let mut id_as_bytes: [u8; 4] = [0; 4];
        id_as_bytes.copy_from_slice(&header[beggining..beggining + MSG_ID_FIELD_LEN]);
        Ok(u32::from_le_bytes(id_as_bytes))
    }

    fn response_as_bytes(resp: Box<dyn Response>) -> Vec<u8> {
        let mut as_bytes = Vec::with_capacity(MSG_HEADER_LEN);
        for &ch in SKEY.iter() {
            as_bytes.push(ch);
        }
        as_bytes.extend(&resp.id().to_le_bytes());

        let payload = resp.payload();
        as_bytes.extend(&(payload.len() as u32).to_le_bytes());
        as_bytes.reserve(payload.len());
        as_bytes.extend_from_slice(&payload[..]);

        as_bytes
    }
}
