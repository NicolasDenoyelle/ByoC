use crate::builder::BTreeBuilder;
use crate::config::{
    ConfigError, ConfigInstance, GenericKey, GenericValue, IntoConfig,
};
use crate::objsafe::DynBuildingBlock;
use crate::BTree;
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
/// use byoc::{BuildingBlock, DynBuildingBlock};
/// use byoc::config::{ConfigInstance, ConfigBuilder};
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

    fn build<'a, K: 'a + GenericKey, V: 'a + GenericValue>(
        self,
    ) -> DynBuildingBlock<'a, K, V> {
        DynBuildingBlock::new(BTree::new(self.capacity), false)
    }
}

impl<K: Ord + Copy, V: Ord> IntoConfig<BTreeConfig>
    for BTreeBuilder<K, V>
{
    fn as_config(&self) -> BTreeConfig {
        BTreeConfig {
            id: String::from(BTreeConfig::id()),
            capacity: self.capacity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BTreeConfig;
    use crate::builder::BTreeBuilder;
    use crate::config::tests::test_config_builder;
    use crate::config::{ConfigError, ConfigInstance};
    use crate::objsafe::DynBuildingBlock;
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
        let btree: DynBuildingBlock<u64, u64> = config.build();
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
    fn test_builder_as_config() {
        let builder = BTreeBuilder::<(), ()>::new(2);
        test_config_builder(builder);
    }
}
