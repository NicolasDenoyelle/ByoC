use crate::builder::{Build, FlushStopperBuilder};
use crate::config::{
    ConfigError, ConfigInstance, ConfigWithTraits, GenericConfig,
    GenericKey, GenericValue, IntoConfig,
};
use crate::{BuildingBlock, FlushStopper};
use serde::{Deserialize, Serialize};

/// Configuration format for [`FlushStopper`](../struct.FlushStopper.html)
/// containers.
///
/// This configuration format is composed of a unique `id` field where the
/// `id` value must be "FlushStopperConfig", and the configuration in toml
/// format of the container to wrap.
///
/// Below is an example of the configuration of a
/// [`FlushStopper`](../struct.FlushStopper.html) wrapping an
/// [`Array`](../struct.Array.html) container.
/// ```
/// use byoc::BuildingBlock;
/// use byoc::builder::Build;
/// use byoc::config::{ConfigBuilder, DynBuildingBlock};
///
/// let config_str = format!("
/// id='FlushStopperConfig'
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
pub struct FlushStopperConfig {
    #[allow(dead_code)]
    id: String,
    container: toml::Value,
}

impl<C, B> IntoConfig<FlushStopperConfig> for FlushStopperBuilder<C, B>
where
    C: ConfigInstance,
    B: IntoConfig<C>,
{
    fn as_config(&self) -> FlushStopperConfig {
        let container_toml_str = self.builder.as_config().to_toml_string();
        let container: toml::value::Value =
            toml::de::from_str(container_toml_str.as_ref()).unwrap();
        FlushStopperConfig {
            id: String::from(FlushStopperConfig::id()),
            container,
        }
    }
}

impl ConfigInstance for FlushStopperConfig {
    fn id() -> &'static str {
        "FlushStopperConfig"
    }

    fn from_toml(value: &toml::Value) -> Result<Self, ConfigError> {
        let toml = toml::to_string(&value).unwrap();
        let cfg: FlushStopperConfig = match toml::from_str(&toml) {
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
    for FlushStopperConfig
where
    K: 'a + GenericKey,
    V: 'a + GenericValue,
{
    fn build(self) -> Box<dyn BuildingBlock<'a, K, V> + 'a> {
        Box::new(FlushStopper::new(
            GenericConfig::from_toml(&self.container).unwrap().build(),
        ))
    }
}

impl ConfigWithTraits for FlushStopperConfig {
    fn is_concurrent(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::FlushStopperConfig;
    use crate::builder::{ArrayBuilder, Build, FlushStopperBuilder};
    use crate::config::tests::test_config_builder;
    use crate::config::{ConfigError, ConfigInstance};
    use crate::BuildingBlock;

    #[test]
    fn test_valid_flush_stopper_config() {
        let array_capacity = 10;
        let config_str = format!(
            "id='FlushStopperConfig'
[container]
id='ArrayConfig'
capacity={}
",
            array_capacity
        );
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        let config = FlushStopperConfig::from_toml(&value).unwrap();
        let container: Box<dyn BuildingBlock<u64, u64>> = config.build();
        assert_eq!(container.capacity(), array_capacity);
    }

    #[test]
    fn test_invalid_flush_stopper_config() {
        let config_str = "id='FlushStopperConfig'
[container]
id='ArrayConfig'
capacity='ten'
"
        .to_string();
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        assert!(matches!(
            FlushStopperConfig::from_toml(&value),
            Err(ConfigError::ConfigFormatError(_))
        ));
    }

    #[test]
    fn test_builder_as_config() {
        let builder = FlushStopperBuilder::new(ArrayBuilder::<()>::new(2));
        test_config_builder(builder);
    }
}
