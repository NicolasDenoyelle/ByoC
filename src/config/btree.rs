use crate::config::{BuildingBlockConfig, ConfigError};
use crate::{BTree, BuildingBlock};
use serde::Deserialize;

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
/// use byoc::builder::traits::Builder;
/// use byoc::config::{BuilderConfig, BuildingBlockConfig};
///
/// let config_str = format!("
/// id = 'BTreeConfig'
/// capacity = 10
/// ");
/// let array: Box<dyn BuildingBlock<u64, u64>> =
///            BuilderConfig::from_str(config_str.as_str())
///            .unwrap()
///            .build();
/// ```
#[derive(Deserialize, Clone)]
pub struct BTreeConfig {
    #[allow(dead_code)]
    id: String,
    capacity: usize,
}

impl BuildingBlockConfig for BTreeConfig {
    fn from_toml(value: toml::Value) -> Result<Self, ConfigError> {
        let toml = toml::to_string(&value).unwrap();
        match toml::from_str(&toml) {
            Err(e) => Err(ConfigError::ConfigFormatError(format!(
                "Invalid BTreeConfig: {}\n{:?}",
                toml, e
            ))),
            Ok(cfg) => Ok(cfg),
        }
    }

    fn build<'a, K, V>(self) -> Box<dyn BuildingBlock<'a, K, V> + 'a>
    where
        K: 'a + Copy + Ord,
        V: 'a + Ord,
    {
        Box::new(BTree::new(self.capacity))
    }
}

#[cfg(test)]
mod tests {
    use super::BTreeConfig;
    use crate::config::{BuildingBlockConfig, ConfigError};
    use crate::BuildingBlock;
    use toml;

    #[test]
    fn test_valid_btree_config() {
        let capacity = 10;
        let config_str =
            format!("id='BTreeConfig'\ncapacity={}", capacity);
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        let config = BTreeConfig::from_toml(value).unwrap();
        assert_eq!(config.capacity, capacity);
        let btree: Box<dyn BuildingBlock<u64, u64>> = config.build();
        assert_eq!(btree.capacity(), capacity);
    }

    #[test]
    fn test_invalid_btree_config() {
        let config_str = format!("id='BTreeConfig'\ncapacity='ten'");
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        assert!(matches!(
            BTreeConfig::from_toml(value),
            Err(ConfigError::ConfigFormatError(_))
        ));
    }
}
