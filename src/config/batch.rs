use crate::config::config::{GenericKey, GenericValue};
use crate::config::{Builder, BuildingBlockConfig};
use crate::{Batch, BuildingBlock};
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct BatchConfig {
    #[allow(dead_code)]
    id: String,
    container: toml::value::Array,
}

impl<'a, K, V> BuildingBlockConfig<'a, K, V> for BatchConfig
where
    K: 'a + GenericKey,
    V: 'a + GenericValue,
{
    fn build(self) -> Box<dyn BuildingBlock<'a, K, V> + 'a> {
        let mut c = Batch::new();
        for cfg in self.container {
            c.append(Builder::from_toml(cfg))
        }
        Box::new(c)
    }
}

#[cfg(test)]
mod tests {
    use crate::config::{BatchConfig, BuildingBlockConfig};
    use crate::BuildingBlock;
    use toml;

    #[test]
    fn test_batch_config() {
        let array_capacity = 10;
        let config_str = format!(
            "id='BatchConfig'
[[container]]
id='ArrayConfig'
capacity={}",
            array_capacity
        );
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        let config: BatchConfig =
            BuildingBlockConfig::<u64, u64>::from_toml(value).unwrap();
        assert_eq!(config.container.len(), 1);
        let container: Box<dyn BuildingBlock<u64, u64>> = config.build();
        assert_eq!(container.capacity(), array_capacity);
    }
}
