use crate::builder::ExclusiveBuilder;
use crate::config::{
    ConfigError, ConfigInstance, GenericConfig, GenericKey, GenericValue,
    IntoConfig,
};
use crate::objsafe::DynBuildingBlock;
use crate::Exclusive;
use serde::{Deserialize, Serialize};

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
/// use byoc::{BuildingBlock, DynBuildingBlock};
/// use byoc::config::{ConfigInstance, ConfigBuilder};
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
///                ConfigBuilder::from_string(config_str.as_str())
///                .unwrap()
///                .build();
/// ```
#[derive(Deserialize, Serialize, Clone)]
pub struct ExclusiveConfig {
    #[allow(dead_code)]
    id: String,
    front: toml::Value,
    back: toml::Value,
}

impl<L, LB, R, RB> IntoConfig<ExclusiveConfig>
    for ExclusiveBuilder<L, LB, R, RB>
where
    LB: IntoConfig<L>,
    RB: IntoConfig<R>,
    L: ConfigInstance,
    R: ConfigInstance,
{
    fn as_config(&self) -> ExclusiveConfig {
        let left_config: L = self.lbuilder.as_config();
        let right_config: R = self.rbuilder.as_config();
        let left_config_str = left_config.to_toml_string();
        let right_config_str = right_config.to_toml_string();
        let front = toml::de::from_str(left_config_str.as_ref()).unwrap();
        let back = toml::de::from_str(right_config_str.as_ref()).unwrap();

        ExclusiveConfig {
            id: String::from(ExclusiveConfig::id()),
            front,
            back,
        }
    }
}

impl ConfigInstance for ExclusiveConfig {
    fn id() -> &'static str {
        "ExclusiveConfig"
    }

    fn from_toml(value: &toml::Value) -> Result<Self, ConfigError> {
        let toml = toml::to_string(&value).unwrap();
        let cfg: ExclusiveConfig = match toml::from_str(&toml) {
            Err(e) => return Err(ConfigError::TomlFormatError(e)),
            Ok(cfg) => cfg,
        };
        match (
            GenericConfig::from_toml(&cfg.front),
            GenericConfig::from_toml(&cfg.back),
        ) {
            (Ok(_), Ok(_)) => Ok(cfg),
            (Ok(_), Err(e)) => Err(e),
            (Err(e), _) => Err(e),
        }
    }

    fn build<'a, K: 'a + GenericKey, V: 'a + GenericValue>(
        self,
    ) -> DynBuildingBlock<'a, K, V> {
        DynBuildingBlock::new(
            Exclusive::new(
                GenericConfig::from_toml(&self.front).unwrap().build(),
                GenericConfig::from_toml(&self.back).unwrap().build(),
            ),
            false,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ExclusiveConfig;
    use crate::builder::{ArrayBuilder, ExclusiveBuilder};
    use crate::config::tests::test_config_builder;
    use crate::config::{ConfigError, ConfigInstance};
    use crate::objsafe::DynBuildingBlock;
    use crate::BuildingBlock;

    #[test]
    fn test_valid_exclusive_config() {
        let array_capacity = 10;
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
        let config = ExclusiveConfig::from_toml(&value).unwrap();
        let container: DynBuildingBlock<u64, u64> = config.build();
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
            ExclusiveConfig::from_toml(&value),
            Err(ConfigError::ConfigFormatError(_))
        ));
    }

    #[test]
    fn test_builder_as_config() {
        let builder = ExclusiveBuilder::new(
            ArrayBuilder::<()>::new(2),
            ArrayBuilder::<()>::new(2),
        );
        test_config_builder(builder);
    }
}
