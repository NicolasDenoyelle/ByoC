use crate::builder::DecoratorBuilder;
use crate::config::{
    ConfigError, ConfigInstance, GenericConfig, GenericKey, GenericValue,
    IntoConfig,
};
use crate::decorator::{Fifo, Lrfu, Lru};
use crate::objsafe::DynBuildingBlock;
use crate::utils::timestamp::{Counter, Timestamp};
use crate::Decorator;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Copy, Clone)]
#[serde(tag = "kind", content = "exponent")]
pub enum DecorationType {
    Lrfu(f32),
    Lru,
    Fifo,
    None,
}

impl Default for DecorationType {
    /// The default value of a [`DecorationType`] is `None`, i.e no decoration.
    fn default() -> Self {
        DecorationType::None
    }
}

/// Configuration format for [`Decorator`](../struct.Decorator.html)
/// containers.
///
/// ! At the moment this configuration cannot be built due to recursion
/// happening at compile time. The configuration format allows for nesting
/// an arbitrary number of policy containers that will result in values generic
/// being wrapped an arbitrary number of time in a policy cell. As a result,
/// serde crate will automatically recurse on nesting policy types until a
/// compile time error occurs. This cannot be explicitly limited at compile
/// time and in a matching configuration. A workaround for this is to allow to
/// set a policy a single time at the top level of the container configuration.
///
/// This configuration format is composed of     
/// * an `id` field where the `id` value must be "DecoratorConfig",
/// * `decorator.kind` field which accept values defined in the [`DecorationType`]
/// enum,
/// * `decorator.exponent` field that sets the floating point value for the
/// [`Lrfu`](../decorator/struct.Lrfu.html) decorator.
///
/// Below is an example of the configuration of a
/// [`Decorator`](../struct.Decorator.html) wrapping an
/// [`Array`](../struct.Array.html) container.
/// ```no_run
/// use byoc::{BuildingBlock, DynBuildingBlock};
/// use byoc::config::{ConfigInstance, ConfigBuilder};
///
/// let config_str = format!("
/// id='DecoratorConfig'
/// decorator.kind='Fifo'
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
pub struct DecoratorConfig {
    #[allow(dead_code)]
    id: String,
    decorator: DecorationType,
    container: toml::Value,
}

impl DecoratorConfig {
    fn from_builder<C: ConfigInstance, B: IntoConfig<C>>(
        builder: &B,
        decorator: DecorationType,
    ) -> Self {
        let container_config_str = builder.as_config().to_toml_string();
        let container: toml::value::Value =
            toml::de::from_str(container_config_str.as_ref()).unwrap();

        DecoratorConfig {
            id: String::from(DecoratorConfig::id()),
            decorator,
            container,
        }
    }
}

impl<C, V, B, T> IntoConfig<DecoratorConfig>
    for DecoratorBuilder<C, V, Lru<T>, B>
where
    C: ConfigInstance,
    B: IntoConfig<C>,
    T: Timestamp,
{
    fn as_config(&self) -> DecoratorConfig {
        DecoratorConfig::from_builder(&self.builder, DecorationType::Lru)
    }
}

impl<C, V, B, T> IntoConfig<DecoratorConfig>
    for DecoratorBuilder<C, V, Lrfu<T>, B>
where
    C: ConfigInstance,
    B: IntoConfig<C>,
    T: Timestamp,
{
    fn as_config(&self) -> DecoratorConfig {
        DecoratorConfig::from_builder(
            &self.builder,
            DecorationType::Lrfu(self.decorator.exponent()),
        )
    }
}

impl<C, V, B> IntoConfig<DecoratorConfig>
    for DecoratorBuilder<C, V, Fifo, B>
where
    C: ConfigInstance,
    B: IntoConfig<C>,
{
    fn as_config(&self) -> DecoratorConfig {
        DecoratorConfig::from_builder(&self.builder, DecorationType::Fifo)
    }
}

impl ConfigInstance for DecoratorConfig {
    fn id() -> &'static str {
        "DecoratorConfig"
    }

    fn from_toml(value: &toml::Value) -> Result<Self, ConfigError> {
        let toml = toml::to_string(&value).unwrap();
        toml::from_str(&toml).map_err(ConfigError::TomlFormatError)
    }

    fn is_concurrent(&self) -> bool {
        GenericConfig::from_toml(&self.container)
            .unwrap()
            .is_concurrent()
    }

    fn build<'a, K: 'a + GenericKey, V: 'a + GenericValue>(
        self,
    ) -> DynBuildingBlock<'a, K, V> {
        match self.decorator {
            DecorationType::Lrfu(exponent) => DynBuildingBlock::new(
                Decorator::new(
                    GenericConfig::from_toml(&self.container)
                        .unwrap()
                        .build(),
                    Lrfu::<Counter>::new(exponent),
                ),
                self.is_concurrent(),
            ),
            DecorationType::Lru => DynBuildingBlock::new(
                Decorator::new(
                    GenericConfig::from_toml(&self.container)
                        .unwrap()
                        .build(),
                    Lru::<Counter>::new(),
                ),
                self.is_concurrent(),
            ),
            DecorationType::Fifo => DynBuildingBlock::new(
                Decorator::new(
                    GenericConfig::from_toml(&self.container)
                        .unwrap()
                        .build(),
                    Fifo::new(),
                ),
                self.is_concurrent(),
            ),
            DecorationType::None => {
                GenericConfig::from_toml(&self.container).unwrap().build()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DecoratorConfig;
    use crate::builder::{ArrayBuilder, DecoratorBuilder};
    use crate::config::tests::test_config_builder;
    use crate::config::{ConfigError, ConfigInstance};
    use crate::decorator::Fifo;
    use crate::objsafe::DynBuildingBlock;
    use crate::BuildingBlock;

    #[test]
    fn test_valid_decorator_config() {
        let array_capacity = 10;
        let config_str = format!(
            "
id='DecoratorConfig'
decorator.kind='Lrfu'
decorator.exponent=0.5
[container]
id='ArrayConfig'
capacity={}
",
            array_capacity
        );
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        let config = DecoratorConfig::from_toml(&value).unwrap();
        let container: DynBuildingBlock<u64, u64> = config.build();
        assert_eq!(container.capacity(), array_capacity);
    }

    #[test]
    fn test_invalid_decorator_config() {
        let config_str = "
id='DecoratorConfig'
decorator.kind='LRF'
decorator.exponent=0.5
[container]
id='ArrayConfig'
capacity=10
"
        .to_string();
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        assert!(matches!(
            DecoratorConfig::from_toml(&value),
            Err(ConfigError::TomlFormatError(_))
        ));
    }

    #[test]
    fn test_builder_as_config() {
        let builder = DecoratorBuilder::<_, (), _, _>::new(
            ArrayBuilder::<()>::new(2),
            Fifo::new(),
        );
        test_config_builder(builder);
    }
}
