use crate::config::BuildingBlockConfig;
#[cfg(not(feature = "tempfile"))]
use crate::streams::VecStream;
use crate::streams::{
    FileStream, StreamBase, StreamFactory, TempFileStreamFactory,
};
use crate::{BuildingBlock, Compressor};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Deserialize, Clone)]
pub struct CompressorConfig {
    #[allow(dead_code)]
    id: String,
    filename: Option<String>,
    capacity: usize,
}

impl<'a, K, V> BuildingBlockConfig<'a, K, V> for CompressorConfig
where
    K: 'a + Serialize + DeserializeOwned + Eq,
    V: 'a + Serialize + DeserializeOwned + Ord,
{
    fn build(self) -> Box<dyn BuildingBlock<'a, K, V> + 'a> {
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
    use crate::config::BuildingBlockConfig;
    use crate::BuildingBlock;
    use toml;

    #[test]
    fn test_compressor_config() {
        let capacity = 10;
        let config_str =
            format!("id='CompressorConfig'\ncapacity={}", capacity);
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        let config: CompressorConfig =
            BuildingBlockConfig::<u64, u64>::from_toml(value).unwrap();
        assert_eq!(config.capacity, capacity);
        let container: Box<dyn BuildingBlock<u64, u64>> = config.build();
        assert_eq!(container.capacity(), capacity);
    }
}
