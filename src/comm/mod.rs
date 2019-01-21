use std::{
    net::TcpListener,
    sync::{Arc, RwLock},
    thread,
};

use crate::config;

mod errors;
mod connection;
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
            let conn_handler = connection::Handler::new(
                connection::Context::new(conn_count),
                self.req_dispatcher.clone());
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
