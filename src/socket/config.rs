use super::{
    ServerThreadBuilder, ServerThreadHandle, SocketClient, SocketServer,
};
use crate::config::{
    BuildingBlockConfig, ConfigError, GenericConfig, GenericKey,
    GenericValue,
};
use crate::BuildingBlock;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::net::ToSocketAddrs;
use std::time::Duration;

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

fn serde_default_timeout() -> u64 {
    100u64
}

#[derive(Deserialize)]
pub struct SocketServerConfig {
    #[allow(dead_code)]
    id: String,
    address: String,
    #[serde(default = "serde_default_timeout")]
    timeout_ms: u64,
    container: toml::Value,
}

impl SocketServerConfig {
    pub fn from_toml(value: toml::Value) -> Result<Self, ConfigError> {
        let toml = toml::to_string(&value).unwrap();
        let cfg: SocketServerConfig = match toml::from_str(&toml) {
            Err(e) => return Err(ConfigError::TomlFormatError(e)),
            Ok(cfg) => cfg,
        };

        // Check address is valid.
        if let Err(_) = cfg.address.to_socket_addrs() {
            Err(ConfigError::ConfigFormatError(format!(
                "Invalid SocketServerConfig address {}",
                cfg.address
            )))?;
        }

        match GenericConfig::from_toml(cfg.container.clone()) {
            Ok(_) => Ok(cfg),
            Err(e) => Err(e),
        }
    }

    pub fn build<'a, K, V>(
        self,
    ) -> SocketServer<'a, K, V, Box<dyn BuildingBlock<'a, K, V> + 'a>>
    where
        K: 'a + GenericKey,
        V: 'a + GenericValue,
    {
        SocketServer::new(
            self.address,
            GenericConfig::from_toml(self.container).unwrap().build(),
        )
        .unwrap()
        .with_timeout(Some(Duration::from_millis(self.timeout_ms)))
    }

    pub fn spawn<'a, K, V>(self) -> ServerThreadHandle
    where
        K: 'static + GenericKey,
        V: 'static + GenericValue,
    {
        let container: Box<dyn BuildingBlock<'static, K, V> + 'static> =
            GenericConfig::from_toml(self.container).unwrap().build();

        ServerThreadBuilder::new(self.address, container)
            .with_timeout(Duration::from_millis(self.timeout_ms))
            .spawn()
            .unwrap()
    }
}
