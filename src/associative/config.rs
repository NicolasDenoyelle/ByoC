use crate::builder::Build;
use crate::config::{
    ConfigError, ConfigInstance, ConfigWithTraits, GenericConfig,
    GenericKey, GenericValue,
};
use crate::{Associative, BuildingBlock};
use serde::Deserialize;
use std::collections::hash_map::DefaultHasher;

/// Configuration format for [`Associative`](../struct.Associative.html)
/// containers.
///
/// This configuration format is composed of an `id` field where the
/// `id` value must be "AssociativeConfig"; and of an
/// [`array of tables`](https://toml.io/en/v1.0.0#array-of-tables)
/// where each "container" table is a container configuration.
///
/// For instance, below is an [`Associative`](../struct.Associative.html)
/// container of two buckets where each bucket is an
/// [`Array`](../struct.Array.html) container. See
/// [`ArrayConfig`](struct.ArrayConfig.html) for details on Array configuration
/// format.
/// ```
/// use byoc::BuildingBlock;
/// use byoc::builder::Build;
/// use byoc::config::{ConfigBuilder, DynBuildingBlock};
///
/// let config_str = format!("
/// id='AssociativeConfig'
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
#[derive(Deserialize, Clone)]
pub struct AssociativeConfig {
    #[allow(dead_code)]
    id: String,
    container: toml::value::Array,
}

impl ConfigInstance for AssociativeConfig {
    fn from_toml(value: &toml::Value) -> Result<Self, ConfigError> {
        let toml = toml::to_string(&value).unwrap();
        let cfg: AssociativeConfig = match toml::from_str(&toml) {
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

impl<'a, K, V> Build<Box<dyn BuildingBlock<'a, K, V> + 'a>>
    for AssociativeConfig
where
    K: 'a + GenericKey,
    V: 'a + GenericValue,
{
    fn build(self) -> Box<dyn BuildingBlock<'a, K, V> + 'a> {
        Box::new(Associative::new(
            self.container
                .into_iter()
                .map(|cfg| GenericConfig::from_toml(&cfg).unwrap().build())
                .collect(),
            DefaultHasher::new(),
        ))
    }
}

impl ConfigWithTraits for AssociativeConfig {
    fn is_concurrent(&self) -> bool {
        self.container
            .iter()
            .map(|cfg| GenericConfig::from_toml(cfg).unwrap())
            .all(|cfg| cfg.is_concurrent())
    }

    fn is_ordered(&self) -> bool {
        self.container
            .iter()
            .map(|cfg| GenericConfig::from_toml(cfg).unwrap())
            .all(|cfg| cfg.is_ordered())
    }
}

#[cfg(test)]
mod tests {
    use super::AssociativeConfig;
    use crate::builder::Build;
    use crate::config::{ConfigError, ConfigInstance};
    use crate::BuildingBlock;

    #[test]
    fn test_valid_associative_config() {
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
        let config = AssociativeConfig::from_toml(&value).unwrap();
        assert_eq!(config.container.len(), 2);
        let container: Box<dyn BuildingBlock<u64, u64>> = config.build();
        assert_eq!(container.capacity(), array_capacity * 2);
    }

    #[test]
    fn test_invalid_associative_config() {
        let array_capacity = 10;
        let config_str = format!(
            "id='AssociativeConfig'
[[container]]
id='ArrayConfig'
capacity={}
[[container]]
id='ArrayConfg'
capacity={}
",
            array_capacity, array_capacity
        );
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        assert!(matches!(
            AssociativeConfig::from_toml(&value),
            Err(ConfigError::ConfigFormatError(_))
        ));
    }
}
