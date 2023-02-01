use crate::builder::CompressedBuilder;
use crate::config::{
    ConfigError, ConfigInstance, GenericKey, GenericValue, IntoConfig,
};
use crate::objsafe::DynBuildingBlock;
#[cfg(feature = "tempfile")]
use crate::stream::TempFileStreamFactory;
#[cfg(not(feature = "tempfile"))]
use crate::stream::VecStream;
use crate::stream::{FileStream, StreamFactory, VecStreamFactory};
use crate::Compressed;
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
/// use byoc::{BuildingBlock, DynBuildingBlock};
/// use byoc::config::{ConfigInstance, ConfigBuilder};
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
    fn as_config(&self) -> CompressedConfig {
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
    fn as_config(&self) -> CompressedConfig {
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

    fn build<'a, K: 'a + GenericKey, V: 'a + GenericValue>(
        self,
    ) -> DynBuildingBlock<'a, K, V> {
        match self.filename {
            Some(s) => {
                let container =
                    Compressed::new(FileStream::from(&s), self.capacity);
                DynBuildingBlock::new(container, false)
            }
            None => {
                #[cfg(feature = "tempfile")]
                {
                    let mut factory = TempFileStreamFactory {};
                    let container =
                        Compressed::new(factory.create(), self.capacity);
                    DynBuildingBlock::new(container, false)
                }
                #[cfg(not(feature = "tempfile"))]
                {
                    let container = VecStream::new();
                    DynBuildingBlock::new(container, false)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CompressedConfig;
    use crate::builder::CompressedBuilder;
    use crate::config::tests::test_config_builder;
    use crate::config::{ConfigError, ConfigInstance};
    use crate::objsafe::DynBuildingBlock;
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
        let container: DynBuildingBlock<u64, u64> = config.build();
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
    fn test_builder_as_config() {
        let builder =
            CompressedBuilder::<(), _>::new(2, VecStreamFactory {});
        test_config_builder(builder);
    }
}
