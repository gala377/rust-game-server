extern crate glib;

use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;

//use std::thread;

use glib::config::ServerConfig;

fn main() {
    let config = ServerConfig::from_file("config.toml").unwrap();
    println!("read config {}", config);
    let listener = TcpListener::bind(config.to_string()).unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 512];

    stream.read(&mut buffer).unwrap();
    let response = "HTTP/1.1 200 OK\r\n\r\n";

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
