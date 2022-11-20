use crate::builder::{BTreeBuilder, Build};
use crate::config::{
    ConfigError, ConfigInstance, ConfigWithTraits, GenericKey,
    GenericValue, IntoConfig,
};
use crate::{BTree, BuildingBlock};
use serde::{Deserialize, Serialize};

/// Configuration format for [`BTree`](../struct.BTree.html)
/// containers.
///
/// This configuration format is composed of two key/value fields that
/// must be present:      
/// - `id = "BTreeConfig"` and
/// - `capacity = <int>`
///
/// The `id` field must be exactly "BTreeConfig" while the capacity
/// will set the maximum number of key/value pairs that the array can
/// hold.
/// ```
/// use byoc::BuildingBlock;
/// use byoc::builder::Build;
/// use byoc::config::{ConfigBuilder, DynBuildingBlock};
///
/// let config_str = format!("
/// id = 'BTreeConfig'
/// capacity = 10
/// ");
/// let array: DynBuildingBlock<u64, u64> =
///            ConfigBuilder::from_string(config_str.as_str())
///            .unwrap()
///            .build();
/// ```
#[derive(Deserialize, Serialize, Clone)]
pub struct BTreeConfig {
    #[allow(dead_code)]
    id: String,
    capacity: usize,
}

impl ConfigInstance for BTreeConfig {
    fn id() -> &'static str {
        "BTreeConfig"
    }

    fn from_toml(value: &toml::Value) -> Result<Self, ConfigError> {
        let toml = toml::to_string(&value).unwrap();
        toml::from_str(&toml).map_err(|e| {
            ConfigError::ConfigFormatError(format!(
                "Invalid BTreeConfig: {}\n{:?}",
                toml, e
            ))
        })
    }
}

impl<'a, K, V> Build<Box<dyn BuildingBlock<'a, K, V> + 'a>> for BTreeConfig
where
    K: 'a + GenericKey,
    V: 'a + GenericValue,
{
    fn build(self) -> Box<dyn BuildingBlock<'a, K, V> + 'a> {
        Box::new(BTree::new(self.capacity))
    }
}

impl ConfigWithTraits for BTreeConfig {}

impl<K: Ord + Copy, V: Ord> IntoConfig<BTreeConfig>
    for BTreeBuilder<K, V>
{
    fn into_config(&self) -> BTreeConfig {
        BTreeConfig {
            id: String::from(BTreeConfig::id()),
            capacity: self.capacity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BTreeConfig;
    use crate::builder::{BTreeBuilder, Build};
    use crate::config::tests::test_config_builder;
    use crate::config::{ConfigError, ConfigInstance};
    use crate::BuildingBlock;

    #[test]
    fn test_valid_btree_config() {
        let capacity = 1008;
        let config_str =
            format!("id='BTreeConfig'\ncapacity={}", capacity);
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        let config = BTreeConfig::from_toml(&value).unwrap();
        assert_eq!(config.capacity, capacity);
        let btree: Box<dyn BuildingBlock<u64, u64>> = config.build();
        assert_eq!(btree.capacity(), capacity);
    }

    #[test]
    fn test_invalid_btree_config() {
        let config_str = "id='BTreeConfig'\ncapacity='ten'".to_string();
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        assert!(matches!(
            BTreeConfig::from_toml(&value),
            Err(ConfigError::ConfigFormatError(_))
        ));
    }

    #[test]
    fn test_builder_into_config() {
        let builder = BTreeBuilder::<(), ()>::new(2);
        test_config_builder(builder);
    }
}
