use crate::config::config::{GenericKey, GenericValue};
use crate::config::{Builder, BuildingBlockConfig};
use crate::{BuildingBlock, Multilevel};
use serde::Deserialize;
use toml;

#[derive(Deserialize, Clone)]
pub struct MultilevelConfig {
    #[allow(dead_code)]
    id: String,
    left: toml::Value,
    right: toml::Value,
}

impl<'a, K, V> BuildingBlockConfig<'a, K, V> for MultilevelConfig
where
    K: 'a + GenericKey,
    V: 'a + GenericValue,
{
    fn build(self) -> Box<dyn BuildingBlock<'a, K, V> + 'a> {
        Box::new(Multilevel::new(
            Builder::from_toml(self.left),
            Builder::from_toml(self.right),
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::config::{BuildingBlockConfig, MultilevelConfig};
    use crate::BuildingBlock;
    use toml;

    #[test]
    fn test_multilevel_config() {
        let array_capacity = 10;
        let config_str = format!(
            "id='MultilevelConfig'
[left]
id='ArrayConfig'
capacity={}
[right]
id='ArrayConfig'
capacity={}
",
            array_capacity, array_capacity
        );
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        let config: MultilevelConfig =
            BuildingBlockConfig::<u64, u64>::from_toml(value).unwrap();
        let container: Box<dyn BuildingBlock<u64, u64>> = config.build();
        assert_eq!(container.capacity(), array_capacity * 2);
    }
}
