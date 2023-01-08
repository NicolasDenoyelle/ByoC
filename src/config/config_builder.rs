use super::{
    ConfigError, ConfigInstance, GenericConfig, GenericKey, GenericValue,
};
use crate::builder::Build;
use crate::config::DynBuildingBlock;
use crate::decorator::config::DecorationType;
use crate::decorator::{Fifo, Lrfu, Lru};
use crate::utils::timestamp::Counter;
use crate::Decorator;
use serde::Serialize;
use toml;

/// `BuildingBlock` builder from a generic configuration.
///
/// This structure is the entry point to build a cache container from a
/// configuration file/string. It is instantiated from a
/// [`toml`](../../toml/index.html) configuration string or file and consumed
/// to produce a cache container. The produced container comes as a
/// [`DynBuildingBlock`] implementing the
/// [`BuildingBlock`](../trait.BuildingBlock.html) trait.
///
/// In order to be valid, a configuration must be in a proper
/// [`toml`](../../toml/index.html) format where the root element is a
/// [`toml`](../../toml/index.html) [`Table`](../../toml/value/type.Table.html).
/// The table must contain an "id" key identifying the type of container to
/// build. Valid container types are enumerated in the
/// [`configs`](configs/index.html) module.
///
/// If one of these condition is not satisfied,
/// a [`ConfigError::ConfigFormatError`] will be returned instead of a valid
/// [`ConfigBuilder`] when instantiating the [`ConfigBuilder`].
/// Finally, the container is attempted to be built against the identified
/// configuration to validate its configuration. At this step, if the
/// configuration is found to be invalid, usually, a
/// [`ConfigError::TomlFormatError`] is returned, but it is up to the config
/// implementer to choose the type of error to return. This error can mean
/// that either the configuration format is not a proper `toml` format or that
/// the configuration did not match the expected configuration format.
/// In either case, the embedded
/// [`toml::de::Error`](../../toml/de/struct.Error.html) will give
/// more information on where the error is found in the configuration.
///
/// ## [`BuildingBlock`](../trait.BuildingBlock.html) Trait
///
/// Because the configuration cannot be known at compile time, [`ConfigBuilder`]
/// objects (with a valid configuration) are
/// built into a [`DynBuildingBlock`] which is merely an alias for
/// [`std::boxed::Box`]`<dyn` [`BuildingBlock`](../trait.BuildingBlock.html)`>`.
/// Every stage of the target cache architecture will be built with the
/// same type (for the same reason). This can penalize deep architectures that
/// will rely heavily on dynamic dispatch.
///
/// ## [`Get`](../trait.Get.html) and [`GetMut`](../trait.GetMut.html) Traits
///
/// [`ConfigBuilder`] cannot be built into an object implementing these traits
/// at the moment. However, it is easy to wrap the [`DynBuildingBlock`] built by
/// a [`ConfigBuilder`] into a multi-level container implementing these traits.
///
/// ## `Concurrent` Trait
///
/// Unfortunately, the [`Concurrent`](../trait.Concurrent.html) trait cannot be
/// built as a dynamic dispatch element because its
/// [`clone()`](../trait.Concurrent.html#tymethod.clone) method requires to
/// know the size of the underlying container at compile time. This is
/// incompatible with the fact that a configuration based container is only
/// known at runtime. However, for some [`Concurrent`](../trait.Concurrent.html)
/// containers, it is safe to copy the dynamic
/// [`BuildingBlock`](../trait.BuildingBlock.html) pointer in a reference
/// counting cell to be used concurrently. For these containers, the
/// [`DynBuildingBlock`] struct obtained from a [`ConfigBuilder`] provides a
/// method
/// [`into_concurrent()`](struct.DynBuildingBlock.html#method.into_concurrent)
/// to return a [`DynConcurrent`](struct.DynConcurrent.html)`<`
/// [`DynBuildingBlock`]`>` that can be safely cloned and used concurrently.
/// The following configurations can be used to build a
/// [`Concurrent`](../trait.Concurrent.html)
/// [`BuildingBlock`](../trait.BuildingBlock.html):
/// * "[`AssociativeConfig`](configs/struct.AssociativeConfig.html)",
/// * "[`SequentialConfig`](configs/struct.SequentialConfig.html)".
///
/// The [`AssociativeConfig`](struct.AssociativeConfig.html) will only build a
/// valid [`Concurrent`](../trait.Concurrent.html)
/// [`BuildingBlock`](../trait.BuildingBlock.html) if the children containers
/// configurations are themselves valid [`Concurrent`](../trait.Concurrent.html)
/// [`BuildingBlock`](../trait.BuildingBlock.html) configurations.
/// The container at the top of the configuration must be a valid `Concurrent`
/// container to be able to convert it with the
/// [`into_concurrent()`](struct.DynBuildingBlock.html#method.into_concurrent)
/// method.
///
/// ## Container `Policies`
///
/// Containers built from a configuration file can carry only one
/// [`decorator`](../decorator/index.html) on the top container of the configuration.
/// This limitation is due to a recursion issue  happening at compile time.
/// The configuration format with one decorator per container stage would allows
/// for nesting an arbitrary number of decorator containers that will result in
/// values generic being wrapped an arbitrary number of time in a decorator cell.
/// As a result, `serde` crate will automatically recurse on nesting decorator
/// types until a compile time error occurs. This cannot be explicitly limited
/// at compile time and in a matching configuration. A workaround for this is
/// to allow to set a decorator a single time at the top level of the container
/// configuration.
///
/// At the moment only three policies are supported via a key/value attribute
/// at the top of the configuration:
/// * `decorator.kind='Fifo'` the [`Fifo`](../decorator/struct.Fifo.html) policy,
/// * `decorator.kind='Lru'` the [`Lru`](../decorator/struct.Lru.html) policy,
/// * `decorator.kind='Lrfu'` the [`Lrfu`](../decorator/struct.Lrfu.html) policy,
///
/// where the `exponent` attribute `Lrfu` last policy can be configured with
/// an additional key/value attribute: `decorator.exponent=<value>`.
///
/// ## Examples
///
/// Here is an example of how to build the container described in the
/// [`builder`](../builder/index.html) module from a configuration:
///
/// ```
/// use byoc::BuildingBlock;
/// use byoc::builder::Build;
/// use byoc::config::{ConfigBuilder, DynBuildingBlock};
///
/// let config_str = format!("
/// id='SequentialConfig'
/// decorator.kind = 'Lru'
/// [container]
/// id='ExclusiveConfig'
/// [container.front]
/// id='ArrayConfig'
/// capacity=10000
/// [container.back]
/// id='ArrayConfig'
/// capacity=1000000
/// ");
/// let mut container: DynBuildingBlock<u64, u64> =
///            ConfigBuilder::from_string(config_str.as_str()).unwrap().build();
/// container.push(vec![(1,2)]);
/// ```
#[derive(Clone, Serialize)]
pub struct ConfigBuilder {
    config: GenericConfig,
    decorator: DecorationType,
}

impl ConfigBuilder {
    /// Build a [`ConfigBuilder`] from a string in the `toml` format.
    pub fn from_string(s: &str) -> Result<Self, ConfigError> {
        ConfigInstance::from_string(s)
    }

    /// Build a [`ConfigBuilder`] from a file in the `toml` format.
    pub fn from_file<P: AsRef<std::path::Path> + std::fmt::Debug>(
        path: P,
    ) -> Result<Self, ConfigError> {
        ConfigInstance::from_file(path)
    }
}

impl ConfigInstance for ConfigBuilder {
    fn from_toml(value: &toml::Value) -> Result<Self, ConfigError> {
        let table = match value {
            toml::Value::Table(t) => t,
            _ => {
                return Err(ConfigError::ConfigFormatError(String::from(
                    "Building Block configuration must be a toml table.",
                )));
            }
        };

        let decorator = match table.get("decorator") {
            None => DecorationType::None,
            Some(toml::value::Value::Table(t)) => match t.get("kind") {
		None => DecorationType::None,
		Some(toml::value::Value::String(s)) => match s.as_ref() {
		"Fifo" => DecorationType::Fifo,
		"Lru" => DecorationType::Lru,
		"Lrfu" => {
		    match table.get("decorator.Lrfu.exponent") {
			None => DecorationType::Lrfu(1.0),
			Some(&toml::value::Value::Float(f)) => DecorationType::Lrfu(f as f32),
			_ => return Err(ConfigError::ConfigFormatError(format!("Invalid exponent format for decorator {},", s)))
		    }
		},
		_ => return Err(ConfigError::ConfigFormatError(format!("Invalid decorator.kind value {}. Must be one of: Fifo, Lru, Lrfu", s))),
		},
	    _ => return Err(ConfigError::ConfigFormatError(String::from("Invalid decorator attribute. Must be 'decorator.kind' or 'decorator.Lrfu'."))),
            },
	    _ => return Err(ConfigError::ConfigFormatError(String::from("Invalid decorator TOML type. Must be toml table."))),
        };

        // Make sure the configuration is valid for specific child configs.
        let config = match GenericConfig::from_toml(value) {
            Err(e) => return Err(e),
            Ok(c) => c,
        };

        Ok(ConfigBuilder { config, decorator })
    }
}

impl<'a, K, V> Build<DynBuildingBlock<'a, K, V>> for ConfigBuilder
where
    K: 'a + GenericKey,
    V: 'a + GenericValue,
{
    fn build(self) -> DynBuildingBlock<'a, K, V> {
        let has_concurrent_trait = self.config.has_concurrent_trait;
        let build = match self.decorator {
            DecorationType::None => self.config.build(),
            DecorationType::Fifo => {
                Box::new(Decorator::new(self.config.build(), Fifo::new()))
            }
            DecorationType::Lru => Box::new(Decorator::new(
                self.config.build(),
                Lru::<Counter>::new(),
            )),
            DecorationType::Lrfu(e) => Box::new(Decorator::new(
                self.config.build(),
                Lrfu::<Counter>::new(e),
            )),
        };
        DynBuildingBlock::new(build, has_concurrent_trait)
    }
}

#[cfg(test)]
mod tests {
    use crate::builder::Build;
    use crate::config::{
        ConfigBuilder, ConfigError, DynBuildingBlock, DynConcurrent,
    };
    use crate::tests::test_concurrent;
    use crate::BuildingBlock;

    #[test]
    fn test_generic_config() {
        let capacity = 10;
        let config_str =
            format!("id=\"ArrayConfig\"\ncapacity={}", capacity);
        let array: DynBuildingBlock<u64, u64> =
            ConfigBuilder::from_string(config_str.as_str())
                .unwrap()
                .build();
        assert_eq!(array.capacity(), capacity);
    }

    #[test]
    fn test_invalid_id_config() {
        let config_str = "id=\"Array\"\ncapacity=10".to_string();
        assert!(matches!(
            ConfigBuilder::from_string(config_str.as_str()),
            Err(ConfigError::ConfigFormatError(_))
        ));
    }

    #[test]
    fn test_fifo_config() {
        let capacity = 10;
        let config_str = format!(
            "
id='ArrayConfig'
capacity={}
decorator.kind='Fifo'
",
            capacity
        );
        let array: DynBuildingBlock<u16, u32> =
            ConfigBuilder::from_string(config_str.as_str())
                .unwrap()
                .build();
        assert_eq!(array.capacity(), capacity);
        assert!(matches!(
            array.into_concurrent(),
            Err(ConfigError::UnsupportedTraitError(_))
        ));
    }

    #[test]
    fn test_lrfu_config() {
        let capacity = 10;
        let config_str = format!(
            "
id='ArrayConfig'
capacity={}
decorator.kind='Lrfu'
decorator.Lrfu.exponent=0.5
",
            capacity
        );
        let array: DynBuildingBlock<u64, u64> =
            ConfigBuilder::from_string(config_str.as_str())
                .unwrap()
                .build();
        assert_eq!(array.capacity(), capacity);
    }

    #[test]
    fn test_invalid_concurrent() {
        let capacity = 10;
        let config_str = format!(
            "
id='ArrayConfig'
capacity={}
",
            capacity
        );
        let container: DynBuildingBlock<u64, u64> =
            ConfigBuilder::from_string(config_str.as_str())
                .unwrap()
                .build();
        assert!(matches!(
            container.into_concurrent(),
            Err(ConfigError::UnsupportedTraitError(_))
        ));
    }

    #[test]
    fn test_valid_concurrent() {
        let capacity = 10;
        let config_str = format!(
            "
id='SequentialConfig'
[container]
id='ArrayConfig'
capacity={}
",
            capacity
        );

        let container: DynConcurrent<DynBuildingBlock<u16, u32>> =
            ConfigBuilder::from_string(config_str.as_str())
                .unwrap()
                .build()
                .into_concurrent()
                .unwrap();
        assert_eq!(container.capacity(), capacity);
        test_concurrent(container, 64);
    }
}
