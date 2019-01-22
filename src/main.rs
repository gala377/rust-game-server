use std::str::FromStr;
use std::string::String;

use glib::comm::Server;

fn main() {
    let mut serv = Server::new(String::from_str("config.toml").unwrap());
    serv.run();
}
