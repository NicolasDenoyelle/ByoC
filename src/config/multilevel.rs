use crate::config::{
    BuildingBlockConfig, ConfigError, GenericConfig, GenericKey,
    GenericValue,
};
use crate::{BuildingBlock, Multilevel};
use serde::Deserialize;
use toml;

/// Configuration format for [`Multilevel`](../struct.Multilevel.html)
/// containers.
///
/// This configuration format is composed of an `id` field where the
/// `id` value must be "MultilevelConfig"; and of two
/// [`toml tables`](https://toml.io/en/v1.0.0#table) `left` and `right`
/// where representing respectively the first tier of storage where element are
/// accessed and the second tier of storage where popped elements from the first
/// tier go.
///
/// For instance, below is a [`Multilevel`](../struct.Multilevel.html)
/// container of two buckets where each tier is an
/// [`Array`](../struct.Array.html) container. See
/// [`ArrayConfig`](struct.ArrayConfig.html) for details on Array configuration
/// format.
/// ```
/// use byoc::BuildingBlock;
/// use byoc::builder::traits::Builder;
/// use byoc::config::{BuilderConfig, BuildingBlockConfig};
///
/// let config_str = format!("
/// id='MultilevelConfig'
/// [left]
/// id='ArrayConfig'
/// capacity=10
/// [right]
/// id='ArrayConfig'
/// capacity=10
/// ");
/// let container: Box<dyn BuildingBlock<u64, u64>> =
///                BuilderConfig::from_str(config_str.as_str())
///                .unwrap()
///                .build();
/// ```
#[derive(Deserialize, Clone)]
pub struct MultilevelConfig {
    #[allow(dead_code)]
    id: String,
    left: toml::Value,
    right: toml::Value,
}

impl BuildingBlockConfig for MultilevelConfig {
    fn build<'a, K, V>(self) -> Box<dyn BuildingBlock<'a, K, V> + 'a>
    where
        K: 'a + GenericKey,
        V: 'a + GenericValue,
    {
        Box::new(Multilevel::new(
            GenericConfig::from_toml(self.left).unwrap().build(),
            GenericConfig::from_toml(self.right).unwrap().build(),
        ))
    }

    fn from_toml(value: toml::Value) -> Result<Self, ConfigError> {
        let toml = toml::to_string(&value).unwrap();
        let cfg: MultilevelConfig = match toml::from_str(&toml) {
            Err(e) => return Err(ConfigError::TomlFormatError(e)),
            Ok(cfg) => cfg,
        };
        match (
            GenericConfig::from_toml(cfg.left.clone()),
            GenericConfig::from_toml(cfg.right.clone()),
        ) {
            (Ok(_), Ok(_)) => Ok(cfg),
            (Ok(_), Err(e)) => Err(e),
            (Err(e), _) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::{
        BuildingBlockConfig, ConfigError, MultilevelConfig,
    };
    use crate::BuildingBlock;
    use toml;

    #[test]
    fn test_valid_multilevel_config() {
        let array_capacity = 10;
        let config_str = format!(
            "id='MultilevelConfig'
[left]
id='ArrayConfig'
capacity={}
[right]
id='ArrayConfig'
capacity={}
",
            array_capacity, array_capacity
        );
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        let config = MultilevelConfig::from_toml(value).unwrap();
        let container: Box<dyn BuildingBlock<u64, u64>> = config.build();
        assert_eq!(container.capacity(), array_capacity * 2);
    }

    #[test]
    fn test_invalid_multilevel_config() {
        let config_str = format!(
            "id='MultilevelConfig'
[left]
id='ArrayConfig'
capacity=10
[right]
id='ArrayConfig'
toto='titi'
"
        );
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        assert!(matches!(
            MultilevelConfig::from_toml(value),
            Err(ConfigError::ConfigFormatError(_))
        ));
    }
}
