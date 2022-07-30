use crate::config::{
    BuildingBlockConfig, ConfigError, GenericConfig, GenericKey,
    GenericValue,
};
use crate::{Associative, BuildingBlock};
use serde::Deserialize;
use std::collections::hash_map::DefaultHasher;
use toml;

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
/// use byoc::builder::traits::Builder;
/// use byoc::config::{BuilderConfig, BuildingBlockConfig};
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
/// let container: Box<dyn BuildingBlock<u64, u64>> =
///                BuilderConfig::from_str(config_str.as_str())
///                .unwrap()
///                .build();
/// ```
#[derive(Deserialize, Clone)]
pub struct AssociativeConfig {
    #[allow(dead_code)]
    id: String,
    container: toml::value::Array,
}

impl BuildingBlockConfig for AssociativeConfig {
    fn build<'a, K, V>(self) -> Box<dyn BuildingBlock<'a, K, V> + 'a>
    where
        K: 'a + GenericKey,
        V: 'a + GenericValue,
    {
        Box::new(Associative::new(
            self.container
                .into_iter()
                .map(|cfg| GenericConfig::from_toml(cfg).unwrap().build())
                .collect(),
            DefaultHasher::new(),
        ))
    }

    fn from_toml(value: toml::Value) -> Result<Self, ConfigError> {
        let toml = toml::to_string(&value).unwrap();
        let cfg: AssociativeConfig = match toml::from_str(&toml) {
            Err(e) => return Err(ConfigError::TomlFormatError(e)),
            Ok(cfg) => cfg,
        };
        for toml in cfg.container.clone() {
            match GenericConfig::from_toml(toml) {
                Ok(_) => {}
                Err(e) => return Err(e),
            }
        }
        Ok(cfg)
    }
}

#[cfg(test)]
mod tests {
    use crate::config::{
        AssociativeConfig, BuildingBlockConfig, ConfigError,
    };
    use crate::BuildingBlock;
    use toml;

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
        let config = AssociativeConfig::from_toml(value).unwrap();
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
            AssociativeConfig::from_toml(value),
            Err(ConfigError::ConfigFormatError(_))
        ));
    }
}
