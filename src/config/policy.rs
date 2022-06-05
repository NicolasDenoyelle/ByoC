use crate::config::config::{GenericKey, GenericValue};
use crate::config::{Builder, BuildingBlockConfig};
use crate::policies::{timestamp::Counter, FIFO, LRFU, LRU};
use crate::{BuildingBlock, Policy};
use serde::Deserialize;
use toml;

#[derive(Deserialize, Clone)]
enum PolicyKind {
    LRFU(f32),
    LRU,
    FIFO,
}

#[derive(Deserialize, Clone)]
pub struct PolicyConfig {
    #[allow(dead_code)]
    id: String,
    policy: PolicyKind,
    container: toml::Value,
}

impl<'a, K, V> BuildingBlockConfig<'a, K, V> for PolicyConfig
where
    K: 'a + GenericKey,
    V: 'a + GenericValue,
{
    fn build(self) -> Box<dyn BuildingBlock<'a, K, V> + 'a> {
        match self.policy {
            PolicyKind::LRFU(exponent) => Box::new(Policy::new(
                Builder::from_toml(self.container),
                LRFU::<Counter>::new(exponent),
            )),
            PolicyKind::LRU => Box::new(Policy::new(
                Builder::from_toml(self.container),
                LRU::<Counter>::new(),
            )),
            PolicyKind::FIFO => Box::new(Policy::new(
                Builder::from_toml(self.container),
                FIFO::new(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::{BuildingBlockConfig, PolicyConfig};
    use crate::BuildingBlock;
    use toml;

    #[test]
    fn test_policy_config() {
        let array_capacity = 10;
        let config_str = format!(
            "id='PolicyConfig'
policy='FIFO'
[container]
id='ArrayConfig'
capacity={}
",
            array_capacity
        );
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        let config: PolicyConfig =
            BuildingBlockConfig::<u64, u64>::from_toml(value).unwrap();
        let container: Box<dyn BuildingBlock<u64, u64>> = config.build();
        assert_eq!(container.capacity(), array_capacity);
    }
}
