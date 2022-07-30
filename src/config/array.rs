use crate::config::{BuildingBlockConfig, ConfigError};
use crate::{Array, BuildingBlock};
use serde::Deserialize;

/// Configuration format for [`Array`](../struct.Array.html) containers.
///
/// This configuration format is composed of two key/value fields that
/// must be present:      
/// - `id = "ArrayConfig"` and
/// - `capacity = <int>`
///
/// The `id` field must be exactly "ArrayConfig" while the capacity
/// will set the maximum number of key/value pairs that the array can
/// hold.
/// ```
/// use byoc::BuildingBlock;
/// use byoc::builder::traits::Builder;
/// use byoc::config::{BuilderConfig, BuildingBlockConfig};
///
/// let config_str = format!("
/// id = 'ArrayConfig'
/// capacity = 10
/// ");
/// let array: Box<dyn BuildingBlock<u64, u64>> =
///            BuilderConfig::from_str(config_str.as_str())
///            .unwrap()
///            .build();
/// ```
#[derive(Deserialize, Clone)]
pub struct ArrayConfig {
    #[allow(dead_code)]
    id: String,
    capacity: usize,
}

impl BuildingBlockConfig for ArrayConfig {
    fn from_toml(value: toml::Value) -> Result<Self, ConfigError> {
        let toml = toml::to_string(&value).unwrap();
        match toml::from_str(&toml) {
            Err(e) => Err(ConfigError::ConfigFormatError(format!(
                "Invalid ArrayConfig: {}\n{:?}",
                toml, e
            ))),
            Ok(cfg) => Ok(cfg),
        }
    }

    fn build<'a, K, V>(self) -> Box<dyn BuildingBlock<'a, K, V> + 'a>
    where
        K: 'a + Eq,
        V: 'a + Ord,
    {
        Box::new(Array::new(self.capacity))
    }
}

#[cfg(test)]
mod tests {
    use super::ArrayConfig;
    use crate::config::{BuildingBlockConfig, ConfigError};
    use crate::BuildingBlock;
    use toml;

    #[test]
    fn test_valid_array_config() {
        let capacity = 10;
        let config_str =
            format!("id='ArrayConfig'\ncapacity={}", capacity);
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        let config = ArrayConfig::from_toml(value).unwrap();
        assert_eq!(config.capacity, capacity);
        let array: Box<dyn BuildingBlock<u64, u64>> = config.build();
        assert_eq!(array.capacity(), capacity);
    }

    #[test]
    fn test_invalid_array_config() {
        let config_str = format!("id=''\ncapacity='ten'");
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        assert!(matches!(
            ArrayConfig::from_toml(value),
            Err(ConfigError::ConfigFormatError(_))
        ));
    }
}
