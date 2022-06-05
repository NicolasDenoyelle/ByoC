use crate::config::BuildingBlockConfig;
use crate::{Array, BuildingBlock};
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct ArrayConfig {
    #[allow(dead_code)]
    id: String,
    capacity: usize,
}

impl<'a, K: 'a + Eq, V: 'a + Ord> BuildingBlockConfig<'a, K, V>
    for ArrayConfig
{
    fn build(self) -> Box<dyn BuildingBlock<'a, K, V> + 'a> {
        Box::new(Array::new(self.capacity))
    }
}

#[cfg(test)]
mod tests {
    use super::ArrayConfig;
    use crate::config::BuildingBlockConfig;
    use crate::BuildingBlock;
    use toml;

    #[test]
    fn test_array_config() {
        let capacity = 10;
        let config_str =
            format!("id='ArrayConfig'\ncapacity={}", capacity);
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        let config: ArrayConfig =
            BuildingBlockConfig::<u64, u64>::from_toml(value).unwrap();
        assert_eq!(config.capacity, capacity);
        let array: Box<dyn BuildingBlock<u64, u64>> = config.build();
        assert_eq!(array.capacity(), capacity);
    }
}
