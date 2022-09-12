use super::{BuildingBlockConfig, ConfigError, GenericKey, GenericValue};
use crate::array::config::ArrayConfig;
use crate::associative::config::AssociativeConfig;
use crate::batch::config::BatchConfig;
use crate::btree::config::BTreeConfig;
#[cfg(feature = "compression")]
use crate::compression::config::CompressedConfig;
use crate::config::{DynBuildingBlock, DynOrdered};
use crate::exclusive::config::ExclusiveConfig;
use crate::policy::config::PolicyKind;
use crate::policy::timestamp::Counter;
use crate::policy::{Fifo, Lrfu, Lru};
use crate::profiler::config::ProfilerConfig;
use crate::sequential::config::SequentialConfig;
#[cfg(feature = "stream")]
use crate::stream::config::StreamConfig;
use crate::{BuildingBlock, Policy};
use std::io::Read;
use toml;

/// Configuration ids supported by [`GenericConfig`].
static CONFIGS: [&str; 9] = [
    "ArrayConfig",
    "AssociativeConfig",
    "BatchConfig",
    "BTreeConfig",
    "CompressedConfig",
    "ExclusiveConfig",
    "ProfilerConfig",
    "SequentialConfig",
    "StreamConfig",
];

/// Private entry point to build a container from a generic configuration.
#[derive(Clone)]
pub(crate) struct GenericConfig {
    pub has_concurrent_trait: bool,
    pub has_ordered_trait: bool,
    toml_config: toml::Value,
}

impl GenericConfig {
    /// Attempt to build a specific building block config from a toml object.
    fn into_config<C: BuildingBlockConfig>(
        v: toml::Value,
    ) -> Result<C, ConfigError> {
        C::from_toml(v)
    }

    fn from_config<C: BuildingBlockConfig>(
        v: toml::Value,
    ) -> Result<GenericConfig, ConfigError> {
        let toml_value = v.clone();
        C::from_toml(v).map(move |cfg| GenericConfig {
            has_concurrent_trait: cfg.is_concurrent(),
            has_ordered_trait: cfg.is_ordered(),
            toml_config: toml_value,
        })
    }
}

impl BuildingBlockConfig for GenericConfig {
    /// Build a container from a toml value object representing a configuration.
    /// This function checks that:
    /// * The toml configuration is a toml `Table`,
    /// * The toml configuration contains an "id" field
    /// * The value of the "id" field is a supported value.
    /// * The target configuration identified by "id" is valid.
    fn from_toml(value: toml::Value) -> Result<Self, ConfigError> {
        // Check toml value is a table.
        let table = match value {
            toml::Value::Table(t) => t,
            _ => {
                return Err(ConfigError::ConfigFormatError(String::from(
                    "Building Block configuration must be a toml table.",
                )))
            }
        };

        // Check config contain an 'id' field.
        let id = match table.get("id") {
            None => {
                return Err(ConfigError::ConfigFormatError(String::from(
                    "Configuration must have an 'id' field.",
                )))
            }
            Some(s) => match s.as_str() {
                Some(s) => String::from(s),
                None => {
                    return Err(ConfigError::ConfigFormatError(
                        String::from("Invalid id type, must be a string."),
                    ))
                }
            },
        };

        let value = toml::value::Value::try_from(table).unwrap();

        // Check id field is a valid id and if it is, try to build the
        // associated config.
        match id.as_str() {
            "ArrayConfig" => Self::from_config::<ArrayConfig>(value),
            "AssociativeConfig" => {
                Self::from_config::<AssociativeConfig>(value)
            }
            "BatchConfig" => Self::from_config::<BatchConfig>(value),
            "BTreeConfig" => Self::from_config::<BTreeConfig>(value),
            #[cfg(feature = "compression")]
            "CompressedConfig" => {
                Self::from_config::<CompressedConfig>(value)
            }
            "ExclusiveConfig" => {
                Self::from_config::<ExclusiveConfig>(value)
            }
            "ProfilerConfig" => Self::from_config::<ProfilerConfig>(value),
            "SequentialConfig" => {
                Self::from_config::<SequentialConfig>(value)
            }
            #[cfg(feature = "stream")]
            "StreamConfig" => Self::from_config::<StreamConfig>(value),
            unknown => Err(ConfigError::ConfigFormatError(format!(
                "Invalid container configuration type: {} 
Possible values are: {:?}.",
                unknown, CONFIGS
            ))),
        }
    }

    /// Build the generic config object into an actual container.
    /// At this point we can assume that the checks from `from_toml()`
    /// method have passed. So we can build the configuration.
    fn build<'a, K, V>(self) -> Box<dyn BuildingBlock<'a, K, V> + 'a>
    where
        K: 'a + GenericKey,
        V: 'a + GenericValue,
    {
        let id = self
            .toml_config
            .as_table()
            .unwrap()
            .get("id")
            .unwrap()
            .as_str()
            .unwrap();

        match id {
            "ArrayConfig" => {
                Self::into_config::<ArrayConfig>(self.toml_config)
                    .unwrap()
                    .build()
            }
            "AssociativeConfig" => {
                Self::into_config::<AssociativeConfig>(self.toml_config)
                    .unwrap()
                    .build()
            }
            "BatchConfig" => {
                Self::into_config::<BatchConfig>(self.toml_config)
                    .unwrap()
                    .build()
            }
            "BTreeConfig" => {
                Self::into_config::<BTreeConfig>(self.toml_config)
                    .unwrap()
                    .build()
            }
            #[cfg(feature = "compression")]
            "CompressedConfig" => {
                Self::into_config::<CompressedConfig>(self.toml_config)
                    .unwrap()
                    .build()
            }
            "ExclusiveConfig" => {
                Self::into_config::<ExclusiveConfig>(self.toml_config)
                    .unwrap()
                    .build()
            }
            "ProfilerConfig" => {
                Self::into_config::<ProfilerConfig>(self.toml_config)
                    .unwrap()
                    .build()
            }
            "SequentialConfig" => {
                Self::into_config::<SequentialConfig>(self.toml_config)
                    .unwrap()
                    .build()
            }
            #[cfg(feature = "stream")]
            "StreamConfig" => {
                Self::into_config::<StreamConfig>(self.toml_config)
                    .unwrap()
                    .build()
            }
            unknown => panic!(
                "Invalid container configuration type: {} 
Possible values are: {:?}.",
                unknown, CONFIGS
            ),
        }
    }
}

/// [`BuildingBlock`](../trait.BuildingBlock.html) builder from a generic
/// configuration.
///
/// This structure is the entry point to build a cache from a configuration
/// file. It is instantiated from a [`toml`](../../toml/index.html)
/// configuration string or file and consumed to create a cache container.
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
/// [`Builder`] when instantiating the [`Builder`].
/// Finally, the container is attempted to be built against the identified
/// configuration to validate its configuration. At this step, if the
/// configuration is found to be invalid, usually, a
/// [`ConfigError::TomlFormatError`] is returned, but it is up to the config
/// implementer to choose the type of error to return. This error can mean
/// that either the configuration format is not a proper `toml` format or that
/// the configuration did not match the expected configuration format.
/// In either case, the embedded
/// [`toml::de::Error`](../../toml/de/struct.Error.html) will give
/// more information on where the error is found.
///
/// ## `BuildingBlock` Trait
///
/// Because the configuration cannot be known at compile time, [`Builder`]
/// objects (with a valid configuration) are
/// built into a [`DynBuildingBlock`] which is merely an alias for
/// [`std::boxed::Box`]`<dyn` [`BuildingBlock`](../trait.BuildingBlock.html)`>`.
/// Every stage of the target cache architecture will be built with the
/// same type (for the same reason). This can penalize deep architectures that
/// will rely heavily on dynamic dispatch.
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
/// [`DynBuildingBlock`] struct obtained from a [`Builder`] provides a method
/// [`concurrent()`](struct.DynBuildingBlock.html#method.concurrent) to
/// return a [`DynConcurrent`](struct.DynConcurrent.html)`<`
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
/// [`concurrent()`](struct.DynBuildingBlock.html#method.concurrent) method.
///
/// ## `Ordered` Trait
///
/// If the container being built supports the
/// [`Ordered`](../policy/trait.Ordered.html) trait, it can also be converted
/// into a [`DynOrdered`]`<`[`DynBuildingBlock`]`>` with the
/// [`ordered()`](struct.DynBuildingBlock.html#method.ordered) method
/// of the [`DynBuildingBlock`] object. The container configuration is checked
/// when instantiating a [`Builder`] object. This is at this moment that the
/// [`Builder`] object is marked as being able to bear the
/// [`Ordered`](../policy/trait.Ordered.html) trait. Containers are recursively
/// checked from the top container to see if they carry the trait. For instance,
/// a [`ArrayConfig`](configs/struct.ArrayConfig.html) can be built into a
/// [`DynOrdered`]`<`[`DynBuildingBlock`]`>`,  a
/// [`BTreeConfig`](configs/struct.BTreeConfig.html) cannot be built as such,
/// while a [`SequentialConfig`](configs/struct.SequentialConfig.html) will
/// depend on whether the child container carries the trait or not. We cannot
/// enforce that containers that carry the trait will have a matching
/// configuration feature, but we are trying to achieve as close as possible
/// of a match.
///
/// ## Container `Policies`
///
/// Containers built from a configuration file can carry only one
/// [`policy`](../policy/index.html) on the top container of the configuration.
/// This limitation is due to a recursion issue  happening at compile time.
/// The configuration format with one policy per container stage would allows
/// for nesting an arbitrary number of policy containers that will result in
/// values generic being wrapped an arbitrary number of time in a policy cell.
/// As a result, `serde` crate will automatically recurse on nesting policy
/// types until a compile time error occurs. This cannot be explicitly limited
/// at compile time and in a matching configuration. A workaround for this is
/// to allow to set a policy a single time at the top level of the container
/// configuration.
///
/// At the moment only three policies are supported via a key/value attribute
/// at the top of the configuration:
/// * `policy.kind='Fifo'` the [`Fifo`](../policy/struct.Fifo.html) policy,
/// * `policy.kind='Lru'` the [`Lru`](../policy/struct.Lru.html) policy,
/// * `policy.kind='Lrfu'` the [`Lrfu`](../policy/struct.Lrfu.html) policy,
///
/// where the `exponent` attribute `Lrfu` last policy can be configured with
/// an additional key/value attribute: `policy.exponent=<value>`.
///
/// ## Examples
///
/// See [module documentation](index.html) and [`DynBuildingBlock`]
/// documentation.
#[derive(Clone)]
pub struct Builder {
    config: GenericConfig,
    policy: PolicyKind,
}

impl Builder {
    /// Instantiate a config builder from configuration string describing the
    /// architecture of the cache to build.
    pub fn from_string(s: &str) -> Result<Self, ConfigError> {
        match toml::from_str::<toml::Value>(s) {
            Ok(value) => Self::from_toml(value),
            Err(e) => Err(ConfigError::TomlFormatError(e)),
        }
    }

    /// Instantiate a config builder from a configuration file describing the
    /// architecture of the cache to build.
    pub fn from_file<P: AsRef<std::path::Path> + std::fmt::Debug>(
        path: P,
    ) -> Result<Self, ConfigError> {
        let mut file = match std::fs::File::open(&path) {
            Ok(f) => f,
            Err(e) => return Err(ConfigError::IOError(e)),
        };
        let mut s = String::from("");

        if let Err(e) = file.read_to_string(&mut s) {
            return Err(ConfigError::IOError(e));
        }
        Self::from_string(s.as_str())
    }

    /// Instantiate a config builder from a toml configuration describing the
    /// architecture of the cache to build.
    fn from_toml(value: toml::Value) -> Result<Self, ConfigError> {
        let table = match &value {
            toml::Value::Table(t) => t,
            _ => {
                return Err(ConfigError::ConfigFormatError(String::from(
                    "Building Block configuration must be a toml table.",
                )));
            }
        };

        let policy = match table.get("policy") {
            None => PolicyKind::None,
            Some(toml::value::Value::Table(t)) => match t.get("kind") {
		None => PolicyKind::None,
		Some(toml::value::Value::String(s)) => match s.as_ref() {
		"Fifo" => PolicyKind::Fifo,
		"Lru" => PolicyKind::Lru,
		"Lrfu" => {
		    match table.get("policy.Lrfu.exponent") {
			None => PolicyKind::Lrfu(1.0),
			Some(&toml::value::Value::Float(f)) => PolicyKind::Lrfu(f as f32),
			_ => return Err(ConfigError::ConfigFormatError(format!("Invalid exponent format for policy {},", s)))
		    }
		},
		_ => return Err(ConfigError::ConfigFormatError(format!("Invalid policy.kind value {}. Must be one of: Fifo, Lru, Lrfu", s))),
		},
	    _ => return Err(ConfigError::ConfigFormatError(String::from("Invalid policy attribute. Must be 'policy.kind' or 'policy.Lrfu'."))),
            },
	    _ => return Err(ConfigError::ConfigFormatError(String::from("Invalid policy TOML type. Must be toml table."))),
        };

        // Make sure the configuration is valid for specific child configs.
        let mut config = match GenericConfig::from_toml(value) {
            Err(e) => return Err(e),
            Ok(c) => c,
        };

        // Make sure policy is not used on a container that does not
        // support ordered trait.
        match (policy, config.has_ordered_trait) {
	    (PolicyKind::None, _) => Ok(Builder { config, policy }),
	    (_, true) => {
		config.has_ordered_trait=false;
		Ok(Builder { config, policy })
	    },
	    _ => Err(ConfigError::UnsupportedTraitError(String::from("Cannot use a policy with a top level container that does not support dynamically built Ordered trait.")))
	}
    }

    /// Build this configuration builder into a container.
    pub fn build<'a, K, V>(self) -> DynBuildingBlock<'a, K, V>
    where
        K: 'a + GenericKey,
        V: 'a + GenericValue,
    {
        let has_concurrent_trait = self.config.has_concurrent_trait;
        let has_ordered_trait = self.config.has_ordered_trait;
        let build = match self.policy {
            PolicyKind::None => self.config.build(),
            PolicyKind::Fifo => Box::new(Policy::new(
                DynOrdered::new(self.config.build(), has_concurrent_trait),
                Fifo::new(),
            )),
            PolicyKind::Lru => Box::new(Policy::new(
                DynOrdered::new(self.config.build(), has_concurrent_trait),
                Lru::<Counter>::new(),
            )),
            PolicyKind::Lrfu(e) => Box::new(Policy::new(
                DynOrdered::new(self.config.build(), has_concurrent_trait),
                Lrfu::<Counter>::new(e),
            )),
        };
        DynBuildingBlock::new(
            build,
            has_concurrent_trait,
            has_ordered_trait,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::config::{
        Builder, ConfigError, DynBuildingBlock, DynConcurrent, DynOrdered,
    };
    use crate::tests::{test_concurrent, test_ordered};
    use crate::{Array, BuildingBlock};

    #[test]
    fn test_generic_config() {
        let capacity = Array::<(u64, u64)>::element_size() * 10;
        let config_str =
            format!("id=\"ArrayConfig\"\ncapacity={}", capacity);
        let array: DynBuildingBlock<u64, u64> =
            Builder::from_string(config_str.as_str()).unwrap().build();
        assert_eq!(array.capacity(), capacity);
    }

    #[test]
    fn test_invalid_id_config() {
        let config_str = "id=\"Array\"\ncapacity=10".to_string();
        assert!(matches!(
            Builder::from_string(config_str.as_str()),
            Err(ConfigError::ConfigFormatError(_))
        ));
    }

    #[test]
    fn test_fifo_config() {
        let capacity = Array::<(u16, u32)>::element_size() * 10;
        let config_str = format!(
            "
id='ArrayConfig'
capacity={}
policy.kind='Fifo'
",
            capacity
        );
        let array: DynBuildingBlock<u16, u32> =
            Builder::from_string(config_str.as_str()).unwrap().build();
        assert_eq!(array.capacity(), capacity);
        assert!(matches!(
            array.ordered(),
            Err(ConfigError::UnsupportedTraitError(_))
        ));
    }

    #[test]
    fn test_lrfu_config() {
        let capacity = Array::<(u64, u64)>::element_size() * 10;
        let config_str = format!(
            "
id='ArrayConfig'
capacity={}
policy.kind='Lrfu'
policy.Lrfu.exponent=0.5
",
            capacity
        );
        let array: DynBuildingBlock<u64, u64> =
            Builder::from_string(config_str.as_str()).unwrap().build();
        assert_eq!(array.capacity(), capacity);
        assert!(matches!(
            array.ordered(),
            Err(ConfigError::UnsupportedTraitError(_))
        ));
    }

    #[test]
    fn test_invalid_ordered() {
        let capacity = Array::<(u16, u32)>::element_size() * 10;
        let config_str = format!(
            "
id='BTreeConfig'
capacity={}
policy.kind='Fifo'
",
            capacity
        );
        let builder = Builder::from_string(config_str.as_str());
        assert!(matches!(
            builder,
            Err(ConfigError::UnsupportedTraitError(_))
        ));
    }

    #[test]
    fn test_valid_ordered() {
        let capacity = Array::<(u16, u32)>::element_size() * 10;
        let config_str = format!(
            "
id='ArrayConfig'
capacity={}
",
            capacity
        );
        let builder = Builder::from_string(config_str.as_str()).unwrap();
        let container: DynOrdered<DynBuildingBlock<u16, u32>> =
            builder.build().ordered().unwrap();
        test_ordered(container);
    }

    #[test]
    fn test_invalid_concurrent() {
        let capacity = Array::<(u64, u64)>::element_size() * 10;
        let config_str = format!(
            "
id='ArrayConfig'
capacity={}
",
            capacity
        );
        let container: DynBuildingBlock<u64, u64> =
            Builder::from_string(config_str.as_str()).unwrap().build();
        assert!(matches!(
            container.concurrent(),
            Err(ConfigError::UnsupportedTraitError(_))
        ));
    }

    #[test]
    fn test_valid_concurrent() {
        let capacity = Array::<(u16, u32)>::element_size() * 10;
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
            Builder::from_string(config_str.as_str())
                .unwrap()
                .build()
                .concurrent()
                .unwrap();
        assert_eq!(container.capacity(), capacity);
        test_concurrent(container, 64);
    }
}
