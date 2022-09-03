use crate::config::{BuildingBlockConfig, ConfigError};
#[cfg(feature = "tempfile")]
use crate::stream::TempFileStreamFactory;
#[cfg(not(feature = "tempfile"))]
use crate::stream::VecStream;
use crate::stream::{FileStream, StreamBase, StreamFactory};
use crate::{BuildingBlock, Compressed};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::cmp::Ord;

/// Configuration format for [`Compressed`](../struct.Compressed.html)
/// containers.
///
/// This configuration format is composed of three key/value fields.
/// - `id = "CompressedConfig"` (compulsory)
/// - `capacity = <int>` (compulsory)
/// - `filename = <string>` (optional)
/// The `id` field must be exactly "CompressedConfig".
/// The `capacity` will set the maximum number of key/value pairs that
/// the container can hold.
/// The filename field will use the named file as storage for compressed
/// elements. If filename is not present, compressed elements will be stored
/// in memory.
/// ```
/// use byoc::BuildingBlock;
/// use byoc::builder::Build;
/// use byoc::config::{Builder, DynBuildingBlock};
///
/// let config_str = format!("
/// id = 'CompressedConfig'
/// capacity = 10
/// ");
/// let container: DynBuildingBlock<u64, u64> =
///                Builder::from_string(config_str.as_str())
///                .unwrap()
///                .build();
/// ```
#[derive(Deserialize, Clone)]
pub struct CompressedConfig {
    #[allow(dead_code)]
    id: String,
    filename: Option<String>,
    capacity: usize,
}

impl BuildingBlockConfig for CompressedConfig {
    fn from_toml(value: toml::Value) -> Result<Self, ConfigError> {
        let toml = toml::to_string(&value).unwrap();
        toml::from_str(&toml).map_err(|e| {
            ConfigError::ConfigFormatError(format!(
                "Invalid CompressionConfig: {}\n{:?}",
                toml, e
            ))
        })
    }

    fn is_ordered(&self) -> bool {
        true
    }

    fn build<'a, K, V>(self) -> Box<dyn BuildingBlock<'a, K, V> + 'a>
    where
        K: 'a + Serialize + DeserializeOwned + Ord,
        V: 'a + Serialize + DeserializeOwned + Ord,
    {
        let s: Box<dyn StreamBase> = match self.filename {
            Some(s) => Box::new(FileStream::from(&s)),
            None => {
                #[cfg(feature = "tempfile")]
                {
                    let mut factory = TempFileStreamFactory {};
                    Box::new(factory.create())
                }
                #[cfg(not(feature = "tempfile"))]
                {
                    Box::new(VecStream::new())
                }
            }
        };
        Box::new(Compressed::new(s, self.capacity))
    }
}

#[cfg(test)]
mod tests {
    use super::CompressedConfig;
    use crate::config::{BuildingBlockConfig, ConfigError};
    use crate::BuildingBlock;

    #[test]
    fn test_valid_compressed_config() {
        let capacity = 10;
        let config_str =
            format!("id='CompressedConfig'\ncapacity={}", capacity);
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        let config = CompressedConfig::from_toml(value).unwrap();
        assert_eq!(config.capacity, capacity);
        let container: Box<dyn BuildingBlock<u64, u64>> = config.build();
        assert_eq!(container.capacity(), capacity);
    }

    #[test]
    fn test_invalid_compressed_config() {
        let config_str =
            "id='CompressedConfig'\ncapacity='ten'".to_string();
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        assert!(matches!(
            CompressedConfig::from_toml(value),
            Err(ConfigError::ConfigFormatError(_))
        ));
    }
}
