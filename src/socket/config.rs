use super::{ServerThreadBuilder, ServerThreadHandle, SocketClient};
use crate::builder::{Build, SocketClientBuilder, SocketServerBuilder};
use crate::config::{
    ConfigError, ConfigInstance, GenericConfig, GenericKey, GenericValue,
    IntoConfig,
};
use crate::objsafe::DynBuildingBlock;
use serde::{Deserialize, Serialize};
use std::net::ToSocketAddrs;

/// Configuration to build a `SocketClient`
///
/// This configuration format is composed of a unique `id` field where the
/// `id` value must be "SocketClientConfig", and the address on which to connect
/// the client.
///
/// Below is an example of how to build a
/// [`SocketClient`](../struct.SocketClient.html) from a configuration that
/// connects to a server.
///
/// ```
/// use byoc::{BuildingBlock, DynBuildingBlock};
/// use byoc::config::{ConfigInstance, ConfigBuilder};
/// use byoc::utils::socket::ServerThreadBuilder;
/// use byoc::Array;
///
/// // First we need to make a server to connect a client to.
/// let container = Array::<(i32,i32)>::new(10usize);
/// let address = "localhost:6295";
/// let server = ServerThreadBuilder::new(address, container)
///     .spawn()
///     .unwrap();
/// std::thread::sleep(std::time::Duration::from_millis(50));
///
/// // Here we build the client from a configuration.
/// let config_str = format!("
/// id='SocketClientConfig'
/// address='{}'", address);
///
/// let container: DynBuildingBlock<i32, i32> =
///                ConfigBuilder::from_string(config_str.as_str())
///                .unwrap()
///                .build();
/// assert_eq!(container.capacity(), 10usize);
///
/// // Et voila! Now we can cleanup the server and wrap up.
/// server.stop_and_join().unwrap();
/// ```
#[derive(Deserialize, Serialize, Clone)]
pub struct SocketClientConfig {
    #[allow(dead_code)]
    id: String,
    address: String,
}

impl<A: ToSocketAddrs + ToString> IntoConfig<SocketClientConfig>
    for SocketClientBuilder<A>
{
    fn as_config(&self) -> SocketClientConfig {
        SocketClientConfig {
            id: String::from(SocketClientConfig::id()),
            address: self.address.to_string(),
        }
    }
}

impl ConfigInstance for SocketClientConfig {
    fn id() -> &'static str {
        "SocketClientConfig"
    }

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

    fn build<'a, K: 'a + GenericKey, V: 'a + GenericValue>(
        self,
    ) -> DynBuildingBlock<'a, K, V> {
        DynBuildingBlock::new(
            SocketClient::<K, V>::new(self.address).unwrap(),
            false,
        )
    }
}

/// Configuration to build a `SocketServer` managed by a `ServerThreadHandle`.
///
/// This configuration format is composed of a unique `id` field where the
/// `id` value must be "SocketServerConfig", the `address` on which to listen
/// for a client and and the configuration in toml format of the container
/// to wrap.
///
/// Below is an example of how to build a
/// [`ServerThreadHandle`](../utils/socket/struct.ServerThreadHandle.html)
/// from a configuration that can be connected to a
/// [`SocketClient`](../struct.SocketClient.html).
///
/// ```
/// use byoc::BuildingBlock;
/// use byoc::builder::Build;
/// use byoc::config::{ConfigInstance, configs::SocketServerConfig};
/// use byoc::{Array, SocketClient};
/// use byoc::utils::socket::ServerThreadHandle;
///
/// // Let's make our server.
/// let address = "localhost:6295";
/// let server_str = format!("
/// id='SocketServerConfig'
/// address='{}'
/// [container]
/// id='ArrayConfig'
/// capacity=10
/// ", address);
/// let server_config =
///     SocketServerConfig::from_string(server_str.as_ref()).unwrap();
/// let server: ServerThreadHandle<i32, i32> =
///     server_config.build().unwrap();
/// std::thread::sleep(std::time::Duration::from_millis(50));
///
/// // Now we connect a client to it.
/// let client = SocketClient::<i32,i32>::new(address).unwrap();
///
/// // Et voila! Now we can cleanup the server and wrap up.
/// server.stop_and_join().unwrap();
/// ```
#[derive(Deserialize, Serialize)]
pub struct SocketServerConfig {
    #[allow(dead_code)]
    id: String,
    address: String,
    container: toml::Value,
}

impl SocketServerConfig {
    pub fn from_toml(value: &toml::Value) -> Result<Self, ConfigError> {
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

    pub fn from_string(s: &str) -> Result<Self, ConfigError> {
        match toml::from_str::<toml::Value>(s) {
            Ok(value) => Self::from_toml(&value),
            Err(e) => Err(ConfigError::TomlFormatError(e)),
        }
    }

    fn id() -> &'static str {
        "SocketServerConfig"
    }
}

impl<K, V, A, C, B> From<&SocketServerBuilder<K, V, A, C, B>>
    for SocketServerConfig
where
    A: ToSocketAddrs + ToString,
    B: IntoConfig<C>,
    C: ConfigInstance,
{
    fn from(builder: &SocketServerBuilder<K, V, A, C, B>) -> Self {
        let address = builder.address.to_string();
        let container_toml_str =
            builder.container_builder.as_config().to_toml_string();
        let container: toml::value::Value =
            toml::de::from_str(container_toml_str.as_ref()).unwrap();

        SocketServerConfig {
            id: String::from(SocketServerConfig::id()),
            address,
            container,
        }
    }
}

impl<K, V> Build<std::io::Result<ServerThreadHandle<K, V>>>
    for SocketServerConfig
where
    K: 'static + GenericKey,
    V: 'static + GenericValue,
{
    fn build(self) -> std::io::Result<ServerThreadHandle<K, V>> {
        let container: DynBuildingBlock<'static, K, V> =
            GenericConfig::from_toml(&self.container).unwrap().build();

        ServerThreadBuilder::new(self.address, container).spawn()
    }
}

#[cfg(test)]
mod tests {
    use super::{SocketClientConfig, SocketServerConfig};
    use crate::builder::{Build, SocketClientBuilder};
    use crate::config::tests::test_config_builder;
    use crate::config::{ConfigError, ConfigInstance};
    use crate::objsafe::DynBuildingBlock;
    use crate::socket::{
        ServerThreadBuilder, ServerThreadHandle, SocketClient,
    };
    use crate::tests::{TestKey, TestValue};
    use crate::Array;
    use crate::BuildingBlock;

    #[test]
    fn test_valid_client_config() {
        let capacity = 10usize;
        let container = Array::<(TestKey, TestValue)>::new(capacity);
        let address = "localhost:6291";
        let server = ServerThreadBuilder::new(address, container)
            .spawn()
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(50));

        let client_str = format!(
            "
id='SocketClientConfig'
address='{}'",
            address
        );
        let client_config =
            SocketClientConfig::from_string(client_str.as_ref()).unwrap();
        let client: DynBuildingBlock<TestKey, TestValue> =
            client_config.build();
        assert_eq!(client.capacity(), capacity);
        server.stop_and_join().unwrap();
    }

    #[test]
    fn test_invalid_client_config() {
        let address = "localhost:6292";
        let client_str = format!(
            "
id='SocketClientConfig'
addross='{}'",
            address
        );

        assert!(matches!(
            SocketClientConfig::from_string(client_str.as_ref()),
            Err(ConfigError::ConfigFormatError(_))
        ));
    }

    #[test]
    fn test_invalid_client_address() {
        let address = "locaost..abcd";
        let client_str = format!(
            "
id='SocketClientConfig'
address='{}'",
            address
        );
        assert!(matches!(
            SocketClientConfig::from_string(client_str.as_ref()),
            Err(ConfigError::ConfigFormatError(_))
        ));
    }

    #[test]
    fn test_valid_server_config() {
        let capacity = 10usize;
        let address = "localhost:6293";

        let server_str = format!(
            "
id='SocketServerConfig'
address='{}'
[container]
id='ArrayConfig'
capacity={}
",
            address, capacity
        );
        let server_config =
            SocketServerConfig::from_string(server_str.as_ref()).unwrap();
        let server: ServerThreadHandle<TestKey, TestValue> =
            Build::build(server_config).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(100));

        let client =
            SocketClient::<TestKey, TestValue>::new(address).unwrap();
        assert_eq!(client.capacity(), capacity);
        server.stop_and_join().unwrap();
    }

    #[test]
    fn test_invalid_server_config() {
        let address = "localhost:6294";
        let server_str = format!(
            "
id='SocketServerConfig'
address='{}'
[container]
id='ArrayConf'
capacity=10
",
            address
        );

        assert!(matches!(
            SocketServerConfig::from_string(server_str.as_ref()),
            Err(ConfigError::ConfigFormatError(_))
        ));
    }

    #[test]
    fn test_invalid_server_address() {
        let address = "lucalhost:6294";
        let server_str = format!(
            "
id='SocketServerConfig'
address='{}'
[container]
id='ArrayConfig'
capacity=10
",
            address
        );

        assert!(matches!(
            SocketServerConfig::from_string(server_str.as_ref()),
            Err(ConfigError::ConfigFormatError(_))
        ));
    }

    #[test]
    fn test_socket_client_builder_as_config() {
        let builder = SocketClientBuilder::new("localhost:12345");
        test_config_builder(builder);
    }
}
