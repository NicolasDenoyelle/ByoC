use crate::config::config::{GenericKey, GenericValue};
use crate::config::{Builder, BuildingBlockConfig};
use crate::{Associative, BuildingBlock};
use serde::Deserialize;
use std::collections::hash_map::DefaultHasher;
use toml;

#[derive(Deserialize, Clone)]
pub struct AssociativeConfig {
    #[allow(dead_code)]
    id: String,
    container: toml::value::Array,
}

impl<'a, K, V> BuildingBlockConfig<'a, K, V> for AssociativeConfig
where
    K: 'a + GenericKey,
    V: 'a + GenericValue,
{
    fn build(self) -> Box<dyn BuildingBlock<'a, K, V> + 'a> {
        Box::new(Associative::new(
            self.container
                .into_iter()
                .map(|cfg| Builder::from_toml(cfg))
                .collect(),
            DefaultHasher::new(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::config::{AssociativeConfig, BuildingBlockConfig};
    use crate::BuildingBlock;
    use toml;

    #[test]
    fn test_associative_config() {
        let array_capacity = 10;
        let config_str = format!(
            "id='AssociativeConfig'
[[container]]
id='ArrayConfig'
capacity={}
[[container]]
id='ArrayConfig'
capacity={}
",
            array_capacity, array_capacity
        );
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        let config: AssociativeConfig =
            BuildingBlockConfig::<u64, u64>::from_toml(value).unwrap();
        assert_eq!(config.container.len(), 2);
        let container: Box<dyn BuildingBlock<u64, u64>> = config.build();
        assert_eq!(container.capacity(), array_capacity * 2);
    }
}
