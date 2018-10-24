extern crate glib;

use std::str::FromStr;
use std::string::String;

//use std::thread;

use glib::comm::Server;

fn main() {
    let serv = Server::new(String::from_str("config.toml").unwrap());
    serv.run();
}
