use crate::config::{BuildingBlockConfig, ConfigError};
use crate::{BuildingBlock, Stream};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::stream::TempFileStreamFactory;
#[cfg(not(feature = "tempfile"))]
use crate::stream::VecStreamFactory;

/// Configuration format for [`Stream`](../struct.Stream.html)
/// containers.
///
/// This configuration format is composed of an `id` field where the
/// `id` value must be "StreamConfig", and a `capacity` field setting the
/// maximum amount of elements that can be stored in the container.
/// The byte stream used to store elements will either be a temporary file
/// (if the crate was compiled with the feature `tempfile` enabled) or a
/// vector of bytes.
///
/// Below is an example of the configuration of a
/// [`Stream`](../struct.Stream.html).
/// ```
/// use byoc::BuildingBlock;
/// use byoc::builder::Build;
/// use byoc::config::{Builder, DynBuildingBlock};
///
/// let config_str = format!("
/// id='StreamConfig'
/// capacity=10
/// ");
///
/// let container: DynBuildingBlock<u64, u64> =
///                Builder::from_string(config_str.as_str())
///                .unwrap()
///                .build();
/// ```
#[derive(Deserialize, Clone)]
pub struct StreamConfig {
    #[allow(dead_code)]
    id: String,
    capacity: usize,
}

impl BuildingBlockConfig for StreamConfig {
    fn from_toml(value: toml::Value) -> Result<Self, ConfigError>
    where
        Self: Sized,
    {
        let toml = toml::to_string(&value).unwrap();
        toml::from_str(&toml).map_err(|e| {
            ConfigError::ConfigFormatError(format!(
                "Invalid StreamConfig: {}\n{:?}",
                toml, e
            ))
        })
    }

    fn is_ordered(&self) -> bool {
        true
    }

    fn build<
        'a,
        K: 'a + DeserializeOwned + Serialize + Ord,
        V: 'a + DeserializeOwned + Serialize + Ord,
    >(
        self,
    ) -> Box<dyn BuildingBlock<'a, K, V> + 'a> {
        #[cfg(feature = "tempfile")]
        {
            Box::new(Stream::new(TempFileStreamFactory {}, self.capacity))
        }
        #[cfg(not(feature = "tempfile"))]
        {
            Box::new(Stream::new(VecStreamFactory {}, self.capacity))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StreamConfig;
    use crate::config::{BuildingBlockConfig, ConfigError};
    use crate::BuildingBlock;

    #[test]
    fn test_valid_stream_config() {
        let capacity = 10;
        let config_str =
            format!("id='StreamConfig'\ncapacity={}", capacity);
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        let config = StreamConfig::from_toml(value).unwrap();
        assert_eq!(config.capacity, capacity);
        let container: Box<dyn BuildingBlock<u64, u64>> = config.build();
        assert_eq!(container.capacity(), capacity);
    }

    #[test]
    fn test_invalid_stream_config() {
        let config_str = "id='StreamConfig'\ncapacity='ten'".to_string();
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        assert!(matches!(
            StreamConfig::from_toml(value),
            Err(ConfigError::ConfigFormatError(_))
        ));
    }
}