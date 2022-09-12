use crate::config::{
    BuildingBlockConfig, ConfigError, GenericConfig, GenericKey,
    GenericValue,
};
use crate::{BuildingBlock, Exclusive};
use serde::Deserialize;

/// Configuration format for [`Exclusive`](../struct.Exclusive.html)
/// containers.
///
/// This configuration format is composed of an `id` field where the
/// `id` value must be "ExclusiveConfig"; and of two
/// [`toml tables`](https://toml.io/en/v1.0.0#table) `front` and `back`
/// where representing respectively the first tier of storage where element are
/// accessed and the second tier of storage where popped elements from the first
/// tier go.
///
/// For instance, below is a [`Exclusive`](../struct.Exclusive.html)
/// container of two buckets where each tier is an
/// [`Array`](../struct.Array.html) container. See
/// [`ArrayConfig`](struct.ArrayConfig.html) for details on Array configuration
/// format.
/// ```
/// use byoc::BuildingBlock;
/// use byoc::builder::Build;
/// use byoc::config::{Builder, DynBuildingBlock};
///
/// let config_str = format!("
/// id='ExclusiveConfig'
/// [front]
/// id='ArrayConfig'
/// capacity=10
/// [back]
/// id='ArrayConfig'
/// capacity=10
/// ");
/// let container: DynBuildingBlock<u64, u64> =
///                Builder::from_string(config_str.as_str())
///                .unwrap()
///                .build();
/// ```
#[derive(Deserialize, Clone)]
pub struct ExclusiveConfig {
    #[allow(dead_code)]
    id: String,
    front: toml::Value,
    back: toml::Value,
}

impl BuildingBlockConfig for ExclusiveConfig {
    fn build<'a, K, V>(self) -> Box<dyn BuildingBlock<'a, K, V> + 'a>
    where
        K: 'a + GenericKey,
        V: 'a + GenericValue,
    {
        Box::new(Exclusive::new(
            GenericConfig::from_toml(self.front).unwrap().build(),
            GenericConfig::from_toml(self.back).unwrap().build(),
        ))
    }

    fn is_ordered(&self) -> bool {
        GenericConfig::from_toml(self.front.clone())
            .unwrap()
            .has_ordered_trait
            && GenericConfig::from_toml(self.back.clone())
                .unwrap()
                .has_ordered_trait
    }

    fn from_toml(value: toml::Value) -> Result<Self, ConfigError> {
        let toml = toml::to_string(&value).unwrap();
        let cfg: ExclusiveConfig = match toml::from_str(&toml) {
            Err(e) => return Err(ConfigError::TomlFormatError(e)),
            Ok(cfg) => cfg,
        };
        match (
            GenericConfig::from_toml(cfg.front.clone()),
            GenericConfig::from_toml(cfg.back.clone()),
        ) {
            (Ok(_), Ok(_)) => Ok(cfg),
            (Ok(_), Err(e)) => Err(e),
            (Err(e), _) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ExclusiveConfig;
    use crate::config::{BuildingBlockConfig, ConfigError};
    use crate::{Array, BuildingBlock};

    #[test]
    fn test_valid_exclusive_config() {
        let array_capacity = Array::<(u64, u64)>::element_size() * 10;
        let config_str = format!(
            "id='ExclusiveConfig'
[front]
id='ArrayConfig'
capacity={}
[back]
id='ArrayConfig'
capacity={}
",
            array_capacity, array_capacity
        );
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        let config = ExclusiveConfig::from_toml(value).unwrap();
        let container: Box<dyn BuildingBlock<u64, u64>> = config.build();
        assert_eq!(container.capacity(), array_capacity * 2);
    }

    #[test]
    fn test_invalid_exclusive_config() {
        let config_str = "id='ExclusiveConfig'
[front]
id='ArrayConfig'
capacity=10
[back]
id='ArrayConfig'
toto='titi'
"
        .to_string();
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        assert!(matches!(
            ExclusiveConfig::from_toml(value),
            Err(ConfigError::ConfigFormatError(_))
        ));
    }
}
