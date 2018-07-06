
extern crate GLib;

use std::net::TcpListener;

use GLib::config::ServerConfig;

fn main() {
    let config = ServerConfig::from_file("config.toml").unwrap();
    println!("read config {}", config);
    let listener = TcpListener::bind(config.to_string());

}
