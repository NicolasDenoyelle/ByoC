use crate::builder::Build;
use crate::config::{
    ConfigError, ConfigInstance, ConfigWithTraits, GenericConfig,
    GenericKey, GenericValue,
};
use crate::{Batch, BuildingBlock};
use serde::{Deserialize, Serialize};

/// Configuration format for [`Batch`](../struct.Batch.html)
/// containers.
///
/// This configuration format is composed of an `id` field where the
/// `id` value must be "BatchConfig"; and of an
/// [`array of tables`](https://toml.io/en/v1.0.0#array-of-tables)
/// where each "container" table is a container configuration.
///
/// For instance, below is a [`Batch`](../struct.Batch.html)
/// container composed of two containers where each element is an
/// [`Array`](../struct.Array.html) container. See
/// [`ArrayConfig`](struct.ArrayConfig.html) for details on Array configuration
/// format.
/// ```
/// use byoc::BuildingBlock;
/// use byoc::builder::Build;
/// use byoc::config::{ConfigBuilder, DynBuildingBlock};
///
/// let config_str = format!("
/// id='BatchConfig'
/// [[container]]
/// id='ArrayConfig'
/// capacity=10
/// [[container]]
/// id='ArrayConfig'
/// capacity=10
/// ");
/// let container: DynBuildingBlock<u64, u64> =
///                ConfigBuilder::from_string(config_str.as_str())
///                .unwrap()
///                .build();
/// ```
#[derive(Deserialize, Serialize, Clone)]
pub struct BatchConfig {
    #[allow(dead_code)]
    id: String,
    container: toml::value::Array,
}

impl ConfigWithTraits for BatchConfig {}

impl ConfigInstance for BatchConfig {
    fn id() -> &'static str {
        "BatchConfig"
    }

    fn from_toml(value: &toml::Value) -> Result<Self, ConfigError> {
        let toml = toml::to_string(&value).unwrap();
        let cfg: BatchConfig = match toml::from_str(&toml) {
            Err(e) => return Err(ConfigError::TomlFormatError(e)),
            Ok(cfg) => cfg,
        };
        for toml in cfg.container.clone() {
            match GenericConfig::from_toml(&toml) {
                Ok(_) => {}
                Err(e) => return Err(e),
            }
        }
        Ok(cfg)
    }
}

impl<'a, K, V> Build<Box<dyn BuildingBlock<'a, K, V> + 'a>> for BatchConfig
where
    K: 'a + GenericKey,
    V: 'a + GenericValue,
{
    fn build(self) -> Box<dyn BuildingBlock<'a, K, V> + 'a> {
        let c = self
            .container
            .into_iter()
            .map(|cfg| GenericConfig::from_toml(&cfg).unwrap().build())
            .fold(Batch::new(), |acc, batch| acc.append(batch));
        Box::new(c)
    }
}

#[cfg(test)]
mod tests {
    use super::BatchConfig;
    use crate::builder::Build;
    use crate::config::{ConfigError, ConfigInstance};
    use crate::BuildingBlock;

    #[test]
    fn test_valid_batch_config() {
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
        let config = BatchConfig::from_toml(&value).unwrap();
        assert_eq!(config.container.len(), 1);
        let container: Box<dyn BuildingBlock<u64, u64>> = config.build();
        assert_eq!(container.capacity(), array_capacity);
    }

    #[test]
    fn test_invalid_batch_config() {
        let config_str = "id='BatchConfig'
[[container]]
id='ArrayConfig'
capacity='ten'"
            .to_string();
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        let out = BatchConfig::from_toml(&value);
        assert!(matches!(out, Err(ConfigError::ConfigFormatError(_))));
    }
}
