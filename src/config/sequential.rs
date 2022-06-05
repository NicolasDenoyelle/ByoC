use crate::config::config::{GenericKey, GenericValue};
use crate::config::{Builder, BuildingBlockConfig};
use crate::{BuildingBlock, Sequential};
use serde::Deserialize;
use toml;

#[derive(Deserialize, Clone)]
pub struct SequentialConfig {
    #[allow(dead_code)]
    id: String,
    container: toml::Value,
}

impl<'a, K, V> BuildingBlockConfig<'a, K, V> for SequentialConfig
where
    K: 'a + GenericKey,
    V: 'a + GenericValue,
{
    fn build(self) -> Box<dyn BuildingBlock<'a, K, V> + 'a> {
        Box::new(Sequential::new(Builder::from_toml(self.container)))
    }
}

#[cfg(test)]
mod tests {
    use crate::config::{BuildingBlockConfig, SequentialConfig};
    use crate::BuildingBlock;
    use toml;

    #[test]
    fn test_sequential_config() {
        let array_capacity = 10;
        let config_str = format!(
            "id='SequentialConfig'
[container]
id='ArrayConfig'
capacity={}
",
            array_capacity
        );
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        let config: SequentialConfig =
            BuildingBlockConfig::<u64, u64>::from_toml(value).unwrap();
        let container: Box<dyn BuildingBlock<u64, u64>> = config.build();
        assert_eq!(container.capacity(), array_capacity);
    }
}
