use std::str::FromStr;
use std::string::String;

use glib::comm::Server;
use glib::game::Game;
use glib::agent::Agent;

fn main() {
    let game = Game::new(2, (100, 100));
    let agent = Agent::new(game);
    let mut serv = Server::new(
        String::from_str("config.toml").unwrap(),
        agent,
    );
    serv.run();
}
