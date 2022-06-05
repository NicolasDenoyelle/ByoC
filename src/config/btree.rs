use crate::config::BuildingBlockConfig;
use crate::{BTree, BuildingBlock};
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct BTreeConfig {
    #[allow(dead_code)]
    id: String,
    capacity: usize,
}

impl<'a, K, V> BuildingBlockConfig<'a, K, V> for BTreeConfig
where
    K: 'a + Copy + Ord,
    V: 'a + Ord,
{
    fn build(self) -> Box<dyn BuildingBlock<'a, K, V> + 'a> {
        Box::new(BTree::new(self.capacity))
    }
}

#[cfg(test)]
mod tests {
    use super::BTreeConfig;
    use crate::config::BuildingBlockConfig;
    use crate::BuildingBlock;
    use toml;

    #[test]
    fn test_btree_config() {
        let capacity = 10;
        let config_str =
            format!("id='BTreeConfig'\ncapacity={}", capacity);
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        let config: BTreeConfig =
            BuildingBlockConfig::<u64, u64>::from_toml(value).unwrap();
        assert_eq!(config.capacity, capacity);
        let btree: Box<dyn BuildingBlock<u64, u64>> = config.build();
        assert_eq!(btree.capacity(), capacity);
    }
}
