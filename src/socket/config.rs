use super::{
    ServerThreadBuilder, ServerThreadHandle, SocketClient, SocketServer,
};
use crate::builder::Build;
use crate::config::{
    ConfigError, ConfigInstance, ConfigWithTraits, GenericConfig,
    GenericKey, GenericValue,
};
use crate::BuildingBlock;
use serde::Deserialize;
use std::net::ToSocketAddrs;
use std::time::Duration;

#[derive(Deserialize, Clone)]
pub struct SocketClientConfig {
    #[allow(dead_code)]
    id: String,
    address: String,
}

impl ConfigInstance for SocketClientConfig {
    fn from_toml(value: &toml::Value) -> Result<Self, ConfigError> {
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
}

impl<'a, K, V> Build<Box<dyn BuildingBlock<'a, K, V> + 'a>>
    for SocketClientConfig
where
    K: 'a + GenericKey,
    V: 'a + GenericValue,
{
    fn build(self) -> Box<dyn BuildingBlock<'a, K, V> + 'a> {
        Box::new(SocketClient::new(self.address).unwrap())
    }
}

impl ConfigWithTraits for SocketClientConfig {}

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

impl ConfigInstance for SocketServerConfig {
    fn from_toml(value: &toml::Value) -> Result<Self, ConfigError> {
        let toml = toml::to_string(&value).unwrap();
        let cfg: SocketServerConfig = match toml::from_str(&toml) {
            Err(e) => return Err(ConfigError::TomlFormatError(e)),
            Ok(cfg) => cfg,
        };

        // Check address is valid.
        if cfg.address.to_socket_addrs().is_err() {
            Err(ConfigError::ConfigFormatError(format!(
                "Invalid SocketServerConfig address {}",
                cfg.address
            )))?;
        }

        match GenericConfig::from_toml(&cfg.container) {
            Ok(_) => Ok(cfg),
            Err(e) => Err(e),
        }
    }
}

impl<'a, K, V>
    Build<SocketServer<'a, K, V, Box<dyn BuildingBlock<'a, K, V> + 'a>>>
    for SocketServerConfig
where
    K: 'a + GenericKey,
    V: 'a + GenericValue,
{
    fn build(
        self,
    ) -> SocketServer<'a, K, V, Box<dyn BuildingBlock<'a, K, V> + 'a>>
    {
        SocketServer::new(
            self.address,
            GenericConfig::from_toml(&self.container).unwrap().build(),
        )
        .unwrap()
        .with_timeout(Some(Duration::from_millis(self.timeout_ms)))
    }
}

impl SocketServerConfig {
    pub fn spawn<K, V>(self) -> ServerThreadHandle
    where
        K: 'static + GenericKey,
        V: 'static + GenericValue,
    {
        let container: Box<dyn BuildingBlock<'static, K, V> + 'static> =
            GenericConfig::from_toml(&self.container).unwrap().build();

        ServerThreadBuilder::new(self.address, container)
            .with_timeout(Duration::from_millis(self.timeout_ms))
            .spawn()
            .unwrap()
    }
}
