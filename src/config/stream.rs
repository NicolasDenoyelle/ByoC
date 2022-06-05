use crate::config::BuildingBlockConfig;
use crate::{BuildingBlock, Stream};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::streams::TempFileStreamFactory;
#[cfg(not(feature = "tempfile"))]
use crate::streams::VecStreamFactory;

#[derive(Deserialize, Clone)]
pub struct StreamConfig {
    #[allow(dead_code)]
    id: String,
    capacity: usize,
}

impl<'a, K, V> BuildingBlockConfig<'a, K, V> for StreamConfig
where
    K: 'a + Serialize + DeserializeOwned + Eq,
    V: 'a + Serialize + DeserializeOwned + Ord,
{
    fn build(self) -> Box<dyn BuildingBlock<'a, K, V> + 'a> {
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
    use crate::config::BuildingBlockConfig;
    use crate::BuildingBlock;
    use toml;

    #[test]
    fn test_stream_config() {
        let capacity = 10;
        let config_str =
            format!("id='StreamConfig'\ncapacity={}", capacity);
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        let config: StreamConfig =
            BuildingBlockConfig::<u64, u64>::from_toml(value).unwrap();
        assert_eq!(config.capacity, capacity);
        let container: Box<dyn BuildingBlock<u64, u64>> = config.build();
        assert_eq!(container.capacity(), capacity);
    }
}
