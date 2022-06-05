use crate::config::config::{GenericKey, GenericValue};
use crate::config::{Builder, BuildingBlockConfig};
use crate::{BuildingBlock, Profiler, ProfilerOutputKind};
use serde::Deserialize;
use toml;

#[derive(Deserialize, Clone)]
pub struct ProfilerConfig {
    #[allow(dead_code)]
    id: String,
    name: String,
    output: ProfilerOutputKind,
    container: toml::Value,
}

impl<'a, K, V> BuildingBlockConfig<'a, K, V> for ProfilerConfig
where
    K: 'a + GenericKey,
    V: 'a + GenericValue,
{
    fn build(self) -> Box<dyn BuildingBlock<'a, K, V> + 'a> {
        Box::new(Profiler::new(
            &self.name,
            self.output,
            Builder::from_toml(self.container),
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::config::{BuildingBlockConfig, ProfilerConfig};
    use crate::BuildingBlock;
    use toml;

    #[test]
    fn test_profiler_config() {
        let array_capacity = 10;
        let config_str = format!(
            "id='ProfilerConfig'
name='test_profiler'
output='None'
[container]
id='ArrayConfig'
capacity={}
",
            array_capacity
        );
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        let config: ProfilerConfig =
            BuildingBlockConfig::<u64, u64>::from_toml(value).unwrap();
        let container: Box<dyn BuildingBlock<u64, u64>> = config.build();
        assert_eq!(container.capacity(), array_capacity);
    }
}
