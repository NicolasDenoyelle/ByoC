use crate::builder::{Build, CompressedBuilder};
use crate::config::{
    ConfigError, ConfigInstance, ConfigWithTraits, GenericKey,
    GenericValue, IntoConfig,
};
#[cfg(feature = "tempfile")]
use crate::stream::TempFileStreamFactory;
#[cfg(not(feature = "tempfile"))]
use crate::stream::VecStream;
use crate::stream::{
    FileStream, StreamBase, StreamFactory, VecStreamFactory,
};
use crate::{BuildingBlock, Compressed};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
#[cfg(feature = "tempfile")]
use tempfile::NamedTempFile;

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
/// use byoc::config::{ConfigBuilder, DynBuildingBlock};
///
/// let config_str = format!("
/// id = 'CompressedConfig'
/// capacity = 10
/// ");
/// let container: DynBuildingBlock<u64, u64> =
///                ConfigBuilder::from_string(config_str.as_str())
///                .unwrap()
///                .build();
/// ```
#[derive(Deserialize, Serialize, Clone)]
pub struct CompressedConfig {
    #[allow(dead_code)]
    id: String,
    filename: Option<String>,
    capacity: usize,
}

impl<T> IntoConfig<CompressedConfig>
    for CompressedBuilder<T, VecStreamFactory>
where
    T: Serialize + DeserializeOwned,
{
    fn into_config(&self) -> CompressedConfig {
        CompressedConfig {
            id: String::from(CompressedConfig::id()),
            filename: None,
            capacity: self.capacity,
        }
    }
}

#[cfg(feature = "tempfile")]
impl<T> IntoConfig<CompressedConfig>
    for CompressedBuilder<T, TempFileStreamFactory>
{
    fn into_config(&self) -> CompressedConfig {
        let tmp =
            NamedTempFile::new().expect("Failed to create temporary file");
        let tmp_name =
            tmp.path().to_str().expect("Invalid temporary file name");
        CompressedConfig {
            id: String::from(CompressedConfig::id()),
            filename: Some(String::from(tmp_name)),
            capacity: self.capacity,
        }
    }
}

impl ConfigInstance for CompressedConfig {
    fn id() -> &'static str {
        "CompressedConfig"
    }

    fn from_toml(value: &toml::Value) -> Result<Self, ConfigError> {
        let toml = toml::to_string(&value).unwrap();
        toml::from_str(&toml).map_err(|e| {
            ConfigError::ConfigFormatError(format!(
                "Invalid CompressionConfig: {}\n{:?}",
                toml, e
            ))
        })
    }
}

impl<'a, K, V> Build<Box<dyn BuildingBlock<'a, K, V> + 'a>>
    for CompressedConfig
where
    K: 'a + GenericKey,
    V: 'a + GenericValue,
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
        Box::new(Compressed::new(s, self.capacity))
    }
}

impl ConfigWithTraits for CompressedConfig {
    fn is_ordered(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::CompressedConfig;
    use crate::builder::{Build, CompressedBuilder};
    use crate::config::tests::test_config_builder;
    use crate::config::{ConfigError, ConfigInstance};
    use crate::stream::VecStreamFactory;
    use crate::BuildingBlock;

    #[test]
    fn test_valid_compressed_config() {
        let capacity = 10;
        let config_str =
            format!("id='CompressedConfig'\ncapacity={}", capacity);
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        let config = CompressedConfig::from_toml(&value).unwrap();
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
            CompressedConfig::from_toml(&value),
            Err(ConfigError::ConfigFormatError(_))
        ));
    }

    #[test]
    fn test_builder_into_config() {
        let builder =
            CompressedBuilder::<(), _>::new(2, VecStreamFactory {});
        test_config_builder(builder);
    }
}
