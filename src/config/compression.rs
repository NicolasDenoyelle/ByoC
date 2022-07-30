use crate::config::{BuildingBlockConfig, ConfigError};
#[cfg(not(feature = "tempfile"))]
use crate::streams::VecStream;
use crate::streams::{
    FileStream, StreamBase, StreamFactory, TempFileStreamFactory,
};
use crate::{BuildingBlock, Compressor};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::cmp::{Eq, Ord};

/// Configuration format for [`Compressor`](../struct.Compressor.html)
/// containers.
///
/// This configuration format is composed of three key/value fields.
/// - `id = "CompressorConfig"` (compulsory)
/// - `capacity = <int>` (compulsory)
/// - `filename = <string>` (optional)
/// The `id` field must be exactly "CompressorConfig".
/// The `capacity` will set the maximum number of key/value pairs that
/// the container can hold.
/// The filename field will use the named file as storage for compressed
/// elements. If filename is not present, compressed elements will be stored
/// in memory.
/// ```
/// use byoc::BuildingBlock;
/// use byoc::builder::traits::Builder;
/// use byoc::config::{BuilderConfig, BuildingBlockConfig};
///
/// let config_str = format!("
/// id = 'CompressorConfig'
/// capacity = 10
/// ");
/// let container: Box<dyn BuildingBlock<u64, u64>> =
///                BuilderConfig::from_str(config_str.as_str())
///                .unwrap()
///                .build();
/// ```
#[derive(Deserialize, Clone)]
pub struct CompressorConfig {
    #[allow(dead_code)]
    id: String,
    filename: Option<String>,
    capacity: usize,
}

impl BuildingBlockConfig for CompressorConfig {
    fn from_toml(value: toml::Value) -> Result<Self, ConfigError> {
        let toml = toml::to_string(&value).unwrap();
        match toml::from_str(&toml) {
            Err(e) => Err(ConfigError::ConfigFormatError(format!(
                "Invalid CompressionConfig: {}\n{:?}",
                toml, e
            ))),
            Ok(cfg) => Ok(cfg),
        }
    }

    fn build<'a, K, V>(self) -> Box<dyn BuildingBlock<'a, K, V> + 'a>
    where
        K: 'a + Serialize + DeserializeOwned + Eq,
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
        Box::new(Compressor::new(s, self.capacity))
    }
}

#[cfg(test)]
mod tests {
    use super::CompressorConfig;
    use crate::config::{BuildingBlockConfig, ConfigError};
    use crate::BuildingBlock;
    use toml;

    #[test]
    fn test_valid_compressor_config() {
        let capacity = 10;
        let config_str =
            format!("id='CompressorConfig'\ncapacity={}", capacity);
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        let config = CompressorConfig::from_toml(value).unwrap();
        assert_eq!(config.capacity, capacity);
        let container: Box<dyn BuildingBlock<u64, u64>> = config.build();
        assert_eq!(container.capacity(), capacity);
    }

    #[test]
    fn test_invalid_compressor_config() {
        let config_str = format!("id='CompressorConfig'\ncapacity='ten'");
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        assert!(matches!(
            CompressorConfig::from_toml(value),
            Err(ConfigError::ConfigFormatError(_))
        ));
    }
}
