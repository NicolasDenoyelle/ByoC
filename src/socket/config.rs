use super::SocketClient;
use crate::config::{BuildingBlockConfig, ConfigError};
use crate::BuildingBlock;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::net::ToSocketAddrs;

#[derive(Deserialize, Clone)]
pub struct SocketClientConfig {
    #[allow(dead_code)]
    id: String,
    address: String,
}

impl BuildingBlockConfig for SocketClientConfig {
    fn from_toml(value: toml::Value) -> Result<Self, ConfigError> {
        let toml = toml::to_string(&value).unwrap();
        let out: SocketClientConfig =
            toml::from_str(&toml).map_err(|e| {
                ConfigError::ConfigFormatError(format!(
                    "Invalid SocketClientConfig: {}\n{:?}",
                    toml, e
                ))
            })?;

        // Check address is valid.
        match out.address.to_socket_addrs() {
            Err(_) => Err(ConfigError::ConfigFormatError(format!(
                "Invalid SocketClientConfig address {}",
                out.address
            ))),
            Ok(_) => Ok(out),
        }
    }

    fn build<'a, K, V>(self) -> Box<dyn BuildingBlock<'a, K, V> + 'a>
    where
        K: 'a + DeserializeOwned + Serialize + Clone,
        V: 'a + DeserializeOwned + Serialize,
    {
        Box::new(SocketClient::new(self.address).unwrap())
    }
}
