use crate::builder::SequentialBuilder;
use crate::config::{
    ConfigError, ConfigInstance, GenericConfig, GenericKey, GenericValue,
    IntoConfig,
};
use crate::objsafe::DynBuildingBlock;
use crate::Sequential;
use serde::{Deserialize, Serialize};

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
/// use byoc::{BuildingBlock, DynBuildingBlock};
/// use byoc::config::{ConfigInstance, ConfigBuilder};
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
#[derive(Deserialize, Serialize, Clone)]
pub struct SequentialConfig {
    #[allow(dead_code)]
    id: String,
    container: toml::Value,
}

impl<C, B> IntoConfig<SequentialConfig> for SequentialBuilder<C, B>
where
    C: ConfigInstance,
    B: IntoConfig<C>,
{
    fn as_config(&self) -> SequentialConfig {
        let container_toml_str = self.builder.as_config().to_toml_string();
        let container: toml::value::Value =
            toml::de::from_str(container_toml_str.as_ref()).unwrap();
        SequentialConfig {
            id: String::from(SequentialConfig::id()),
            container,
        }
    }
}

impl ConfigInstance for SequentialConfig {
    fn id() -> &'static str {
        "SequentialConfig"
    }

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

    fn build<'a, K: 'a + GenericKey, V: 'a + GenericValue>(
        self,
    ) -> DynBuildingBlock<'a, K, V> {
        DynBuildingBlock::new(
            Sequential::new(
                GenericConfig::from_toml(&self.container).unwrap().build(),
            ),
            true,
        )
    }

    fn is_concurrent(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::SequentialConfig;
    use crate::builder::{ArrayBuilder, SequentialBuilder};
    use crate::config::tests::test_config_builder;
    use crate::config::{ConfigError, ConfigInstance};
    use crate::objsafe::DynBuildingBlock;
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
        let container: DynBuildingBlock<u64, u64> = config.build();
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

    #[test]
    fn test_builder_as_config() {
        let builder = SequentialBuilder::new(ArrayBuilder::<()>::new(2));
        test_config_builder(builder);
    }
}
