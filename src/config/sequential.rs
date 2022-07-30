use crate::config::{
    BuildingBlockConfig, ConfigError, GenericConfig, GenericKey,
    GenericValue,
};
use crate::{BuildingBlock, Sequential};
use serde::Deserialize;
use toml;

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
/// use byoc::builder::traits::Builder;
/// use byoc::config::{BuilderConfig, BuildingBlockConfig};
///
/// let config_str = format!("
/// id='SequentialConfig'
/// [container]
/// id='ArrayConfig'
/// capacity=10
/// ");
///
/// let container: Box<dyn BuildingBlock<u64, u64>> =
///                BuilderConfig::from_str(config_str.as_str())
///                .unwrap()
///                .build();
/// ```
#[derive(Deserialize, Clone)]
pub struct SequentialConfig {
    #[allow(dead_code)]
    id: String,
    container: toml::Value,
}

impl BuildingBlockConfig for SequentialConfig {
    fn build<'a, K, V>(self) -> Box<dyn BuildingBlock<'a, K, V> + 'a>
    where
        K: 'a + GenericKey,
        V: 'a + GenericValue,
    {
        Box::new(Sequential::new(
            GenericConfig::from_toml(self.container).unwrap().build(),
        ))
    }

    fn from_toml(value: toml::Value) -> Result<Self, ConfigError> {
        let toml = toml::to_string(&value).unwrap();
        let cfg: SequentialConfig = match toml::from_str(&toml) {
            Err(e) => return Err(ConfigError::TomlFormatError(e)),
            Ok(cfg) => cfg,
        };
        match GenericConfig::from_toml(cfg.container.clone()) {
            Ok(_) => Ok(cfg),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::{
        BuildingBlockConfig, ConfigError, SequentialConfig,
    };
    use crate::BuildingBlock;
    use toml;

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
        let config = SequentialConfig::from_toml(value).unwrap();
        let container: Box<dyn BuildingBlock<u64, u64>> = config.build();
        assert_eq!(container.capacity(), array_capacity);
    }

    #[test]
    fn test_invalid_sequential_config() {
        let config_str = format!(
            "id='SequentialConfig'
[container]
id='ArrayConfig'
capacity='ten'
"
        );
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        assert!(matches!(
            SequentialConfig::from_toml(value),
            Err(ConfigError::ConfigFormatError(_))
        ));
    }
}
