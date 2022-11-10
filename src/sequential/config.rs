use crate::builder::Build;
use crate::config::{
    ConfigError, ConfigInstance, ConfigWithTraits, GenericConfig,
    GenericKey, GenericValue,
};
use crate::{BuildingBlock, Sequential};
use serde::Deserialize;

/// Configuration format for [`Sequential`](../struct.Sequential.html)
/// containers.
///
/// This configuration format is composed of a unique `id` field where the
/// `id` value must be "SequentialConfig", and the configuration in toml
/// format of the container to wrap.
///
/// Below is an example of the configuration of a
/// [`Sequential`](../struct.Sequential.html) wrapping an
/// [`Array`](../struct.Array.html) container.
/// ```
/// use byoc::BuildingBlock;
/// use byoc::builder::Build;
/// use byoc::config::{ConfigBuilder, DynBuildingBlock};
///
/// let config_str = format!("
/// id='SequentialConfig'
/// [container]
/// id='ArrayConfig'
/// capacity=10
/// ");
///
/// let container: DynBuildingBlock<u64, u64> =
///                ConfigBuilder::from_string(config_str.as_str())
///                .unwrap()
///                .build();
/// ```
#[derive(Deserialize, Clone)]
pub struct SequentialConfig {
    #[allow(dead_code)]
    id: String,
    container: toml::Value,
}

impl ConfigInstance for SequentialConfig {
    fn from_toml(value: &toml::Value) -> Result<Self, ConfigError> {
        let toml = toml::to_string(&value).unwrap();
        let cfg: SequentialConfig = match toml::from_str(&toml) {
            Err(e) => return Err(ConfigError::TomlFormatError(e)),
            Ok(cfg) => cfg,
        };
        match GenericConfig::from_toml(&cfg.container) {
            Ok(_) => Ok(cfg),
            Err(e) => Err(e),
        }
    }
}

impl<'a, K, V> Build<Box<dyn BuildingBlock<'a, K, V> + 'a>>
    for SequentialConfig
where
    K: 'a + GenericKey,
    V: 'a + GenericValue,
{
    fn build(self) -> Box<dyn BuildingBlock<'a, K, V> + 'a> {
        Box::new(Sequential::new(
            GenericConfig::from_toml(&self.container).unwrap().build(),
        ))
    }
}

impl ConfigWithTraits for SequentialConfig {
    fn is_concurrent(&self) -> bool {
        true
    }

    fn is_ordered(&self) -> bool {
        GenericConfig::from_toml(&self.container)
            .unwrap()
            .has_ordered_trait
    }
}

#[cfg(test)]
mod tests {
    use super::SequentialConfig;
    use crate::builder::Build;
    use crate::config::{ConfigError, ConfigInstance};
    use crate::BuildingBlock;

    #[test]
    fn test_valid_sequential_config() {
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
        let config = SequentialConfig::from_toml(&value).unwrap();
        let container: Box<dyn BuildingBlock<u64, u64>> = config.build();
        assert_eq!(container.capacity(), array_capacity);
    }

    #[test]
    fn test_invalid_sequential_config() {
        let config_str = "id='SequentialConfig'
[container]
id='ArrayConfig'
capacity='ten'
"
        .to_string();
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        assert!(matches!(
            SequentialConfig::from_toml(&value),
            Err(ConfigError::ConfigFormatError(_))
        ));
    }
}
