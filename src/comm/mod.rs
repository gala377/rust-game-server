use super::config;

use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::error::Error;


mod factory;
mod requests;
mod errors;
mod helpers;

use self::factory::Request;

/// Handles incoming connections and dispatches them 
/// to Worker threads.
pub struct Server {
    config: config::ServerConfig,
    listener: TcpListener,
    req_factory: factory::RequestFactory,
}

/// Denotes max approved request and response lenght
const MAX_MESS_LEN: usize = 128;
const SKEY: &str = "RG";

impl Server {

    /// Creates new server instance.
    pub fn new(filename: String) -> Server {
        // todo handle errors properly
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

    fn init_req_factory() -> factory::RequestFactory {
        let mut f = factory::RequestFactory::new();
        if f.register(0, Box::new(|_raw: factory::RequestRaw| {
            Some(Box::new(requests::Hello{}))
        })) {
            f
        } else {
            panic!("Ccould not register builder functions");
        }
    }

    /// Run starts 2 threads. One waiting for connections. 
    /// And the second one handling console input.
    pub fn run(&self) {
        println!("Staring server.");
        for stream in self.listener.incoming() {
            let stream = stream.unwrap();
            println!("New connection established.");  
            self.handle_connection(stream);      
        }
        // todo one controler thread.
    }

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
        self.handle_response(stream);   
    }

    fn read_mess(&self, stream: &mut TcpStream) -> Result<[u8; 512], Box<Error>> {
        let mut buffer = [0; 512];
        match stream.read(&mut buffer) {
            Ok(n) => {
                println!("Read {} bytes. Proceeding.", n);
            },
            Err(err) => return Err(Box::new(err)),
        }
        Ok(buffer)
    }

    fn handle_request(&self, raw: [u8; 512]) {
        match self.req_factory.from_raw(raw) {
            Ok(val) => {
                println!("It works, hello there!");
                let obj: &requests::Hello = helpers::Cast::as_ref(&val).unwrap();
                println!("It works even more!");
                println!("My id is {}", obj.id());
            },
            Err(_) => println!("Unrecognized message"),
        }
    }

    fn handle_response(&self, mut stream: TcpStream) {
        let response = "HTTP/1.1 200 OK\r\n\r\n";

        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }
}