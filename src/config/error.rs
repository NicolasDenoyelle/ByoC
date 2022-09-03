use std::io::Error as IOError;
use std::string::String;
use toml::de::Error as TomlDeError;

/// Error type obtained from an attempt to parse or build a configuration.
#[derive(Debug)]
pub enum ConfigError {
    /// Invalid toml format: the configuration is not valid toml or
    /// the toml string does not match a valid configuration struct.
    TomlFormatError(TomlDeError),
    /// Failure to parse toml to a valid container configuration.
    /// This can be returned if the configuration does not contain
    /// the 'id' field, or is not in a [`toml::value::Table`] format.
    ConfigFormatError(String),
    /// Failure to read a configuration from a file.
    IOError(IOError),
    /// A dynamic trait is attempted to be added to a container but the
    /// container does not support it. The string argument contains the
    /// unsupported trait.
    UnsupportedTraitError(String),
}
