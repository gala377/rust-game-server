extern crate toml;

use std::error::Error;
use std::fmt;

use super::helpers::file;

pub type ConfigResult<T> = Result<T, Box<Error>>;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub address: String,
    pub port: u16,
}

impl Config {
    pub fn from_file(file_name: &str) -> ConfigResult<Config> {
        let content = file::read(file_name)?;
        let config: Config = toml::from_str(content.as_str())?;

        Ok(config)
    }
}

impl ServerConfig {
    pub fn new(address: String, port: u16) -> ServerConfig {
        ServerConfig { address, port }
    }

    pub fn from_file(file_name: &str) -> ConfigResult<ServerConfig> {
        let config = Config::from_file(file_name)?;
        Ok(config.server)
    }
}

impl fmt::Display for ServerConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.address, self.port)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use helpers::file;

    #[test]
    fn creating_config_empty_file_returns_an_error() {
        let file = file::create_temp_with_content("").unwrap();
        assert!(Config::from_file(file.path().to_str().unwrap()).is_err());
    }

    #[test]
    fn creating_server_config_empty_file_returns_an_error() {
        let file = file::create_temp_with_content("").unwrap();
        assert!(ServerConfig::from_file(file.path().to_str().unwrap()).is_err());
    }

    #[test]
    fn config_is_read_properly() {
        let file = file::create_temp_with_content(
            r#"[server]
        address = "10.0.0.1"
        port = 6543"#,
        ).unwrap();
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.server.address, "10.0.0.1");
        assert_eq!(config.server.port, 6543);
    }

    #[test]
    fn server_config_is_read_properly() {
        let file = file::create_temp_with_content(
            r#"[server]
        address = "10.0.0.1"
        port = 6543"#,
        ).unwrap();
        let config = ServerConfig::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.address, "10.0.0.1");
        assert_eq!(config.port, 6543);
    }

    #[test]
    fn server_config_constructs_properly() {
        let config = ServerConfig {
            address: String::from("10.1.1.1"),
            port: 1111,
        };
        assert_eq!(config.address, "10.1.1.1");
        assert_eq!(config.port, 1111);
    }

    #[test]
    fn server_config_to_string_trait_is_implemented_right() {
        let config = ServerConfig {
            address: String::from("127.0.0.1"),
            port: 1234,
        };
        assert_eq!("127.0.0.1:1234", config.to_string());
    }
}
