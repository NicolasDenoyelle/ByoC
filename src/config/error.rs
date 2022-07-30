use std::io::Error as IOError;
use std::string::String;
use toml::de::Error as TomlDeError;

#[derive(Debug)]
pub enum ConfigError {
    TomlFormatError(TomlDeError),
    ConfigFormatError(String),
    IOError(IOError),
}
