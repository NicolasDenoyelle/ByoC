use crate::builder::ArrayBuilder;
use crate::config::{
    ConfigError, ConfigInstance, GenericKey, GenericValue, IntoConfig,
};
use crate::objsafe::DynBuildingBlock;
use crate::Array;
use serde::{Deserialize, Serialize};

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
/// use byoc::{BuildingBlock, DynBuildingBlock};
/// use byoc::config::{ConfigInstance, ConfigBuilder};
///
/// let config_str = format!("
/// id = 'ArrayConfig'
/// capacity = 10
/// ");
/// let array: DynBuildingBlock<u64, u64> =
///            ConfigBuilder::from_string(config_str.as_str())
///            .unwrap()
///            .build();
/// ```
#[derive(Deserialize, Serialize, Clone)]
pub struct ArrayConfig {
    #[allow(dead_code)]
    id: String,
    capacity: usize,
}

impl ConfigInstance for ArrayConfig {
    fn id() -> &'static str {
        "ArrayConfig"
    }

    fn from_toml(value: &toml::Value) -> Result<Self, ConfigError> {
        let toml = toml::to_string(&value).unwrap();
        toml::from_str(&toml).map_err(|e| {
            ConfigError::ConfigFormatError(format!(
                "Invalid ArrayConfig: {}\n{:?}",
                toml, e
            ))
        })
    }

    fn build<'a, K: 'a + GenericKey, V: 'a + GenericValue>(
        self,
    ) -> DynBuildingBlock<'a, K, V> {
        DynBuildingBlock::new(Array::new(self.capacity), false)
    }
}

impl<T> IntoConfig<ArrayConfig> for ArrayBuilder<T> {
    fn as_config(&self) -> ArrayConfig {
        ArrayConfig {
            id: String::from(ArrayConfig::id()),
            capacity: self.capacity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ArrayConfig;
    use crate::builder::ArrayBuilder;
    use crate::config::tests::test_config_builder;
    use crate::config::ConfigInstance;
    use crate::objsafe::DynBuildingBlock;
    use crate::BuildingBlock;

    #[test]
    fn test_valid_array_config() {
        let capacity = 10;
        let config_str =
            format!("id='ArrayConfig'\ncapacity={}", capacity);
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        let config = ArrayConfig::from_toml(&value).unwrap();
        assert_eq!(config.capacity, capacity);
        let array: DynBuildingBlock<u64, u64> = config.build();
        assert_eq!(array.capacity(), capacity);
    }

    #[test]
    fn test_builder_as_config() {
        let builder = ArrayBuilder::<()>::new(10);
        test_config_builder(builder);
    }
}
