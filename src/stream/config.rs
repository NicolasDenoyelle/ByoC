use crate::builder::StreamBuilder;
use crate::config::{
    ConfigError, ConfigInstance, GenericKey, GenericValue, IntoConfig,
};
use crate::objsafe::DynBuildingBlock;
use crate::Stream;
use serde::{Deserialize, Serialize};

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
/// use byoc::{BuildingBlock, DynBuildingBlock};
/// use byoc::config::{ConfigInstance, ConfigBuilder};
///
/// let config_str = format!("
/// id='StreamConfig'
/// capacity=10
/// ");
///
/// let container: DynBuildingBlock<u64, u64> =
///                ConfigBuilder::from_string(config_str.as_str())
///                .unwrap()
///                .build();
/// ```
#[derive(Deserialize, Serialize, Clone)]
pub struct StreamConfig {
    #[allow(dead_code)]
    id: String,
    capacity: usize,
}

impl<T, F> IntoConfig<StreamConfig> for StreamBuilder<T, F> {
    fn as_config(&self) -> StreamConfig {
        StreamConfig {
            id: String::from(StreamConfig::id()),
            capacity: self.capacity,
        }
    }
}

impl ConfigInstance for StreamConfig {
    fn id() -> &'static str {
        "StreamConfig"
    }

    fn from_toml(value: &toml::Value) -> Result<Self, ConfigError> {
        let toml = toml::to_string(&value).unwrap();
        toml::from_str(&toml).map_err(|e| {
            ConfigError::ConfigFormatError(format!(
                "Invalid StreamConfig: {}\n{:?}",
                toml, e
            ))
        })
    }

    fn build<'a, K: 'a + GenericKey, V: 'a + GenericValue>(
        self,
    ) -> DynBuildingBlock<'a, K, V> {
        #[cfg(feature = "tempfile")]
        {
            DynBuildingBlock::new(
                Stream::new(TempFileStreamFactory {}, self.capacity),
                false,
            )
        }
        #[cfg(not(feature = "tempfile"))]
        {
            DynBuildingBlock::new(
                Stream::new(VecStreamFactory {}, self.capacity),
                false,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StreamConfig;
    use crate::builder::StreamBuilder;
    use crate::config::tests::test_config_builder;
    use crate::config::{ConfigError, ConfigInstance};
    use crate::objsafe::DynBuildingBlock;
    use crate::stream::VecStreamFactory;
    use crate::BuildingBlock;

    #[test]
    fn test_valid_stream_config() {
        let capacity = 10;
        let config_str =
            format!("id='StreamConfig'\ncapacity={}", capacity);
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        let config = StreamConfig::from_toml(&value).unwrap();
        assert_eq!(config.capacity, capacity);
        let container: DynBuildingBlock<u64, u64> = config.build();
        assert_eq!(container.capacity(), capacity);
    }

    #[test]
    fn test_invalid_stream_config() {
        let config_str = "id='StreamConfig'\ncapacity='ten'".to_string();
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        assert!(matches!(
            StreamConfig::from_toml(&value),
            Err(ConfigError::ConfigFormatError(_))
        ));
    }

    #[test]
    fn test_builder_as_config() {
        let builder = StreamBuilder::<(), _>::new(VecStreamFactory {}, 2);
        test_config_builder(builder);
    }
}
