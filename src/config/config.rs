#[cfg(feature = "compression")]
use crate::config::CompressorConfig;
#[cfg(feature = "stream")]
use crate::config::StreamConfig;
use crate::config::{
    ArrayConfig, AssociativeConfig, BTreeConfig, BatchConfig,
    BuildingBlockConfig, ConfigError, GenericKey, GenericValue,
    MultilevelConfig, PolicyKind, ProfilerConfig, SequentialConfig,
};
use crate::policies::timestamp::Counter;
use crate::policies::{FIFO, LRFU, LRU};
use crate::private::clone::CloneCell;
use crate::{BuildingBlock, Policy};
use std::io::Read;
use toml;

static CONFIGS: [&'static str; 9] = [
    "ArrayConfig",
    "AssociativeConfig",
    "BatchConfig",
    "BTreeConfig",
    "CompressorConfig",
    "MultilevelConfig",
    "ProfilerConfig",
    "SequentialConfig",
    "StreamConfig",
];

pub struct BuilderConfig {
    config: GenericConfig,
    policy: PolicyKind,
}

impl BuilderConfig {
    /// Build a container from a string configuration.
    pub fn from_str(s: &str) -> Result<Self, ConfigError> {
        match toml::from_str::<toml::Value>(s) {
            Ok(value) => Self::from_toml(value),
            Err(e) => Err(ConfigError::TomlFormatError(e)),
        }
    }

    /// Build a container from a file configuration.
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
        Self::from_str(s.as_str())
    }

    pub fn build_concurrent<'a, K, V>(
        self,
    ) -> Result<
        CloneCell<Box<dyn BuildingBlock<'a, K, V> + 'a>>,
        ConfigError,
    >
    where
        K: 'a + GenericKey,
        V: 'a + GenericValue,
    {
        match self.config.id.as_ref() {
            "SequentialConfig" | "AssociativeConfig"
		=> Ok(CloneCell::new(self.build())),
	    id => Err(ConfigError::ConfigFormatError(format!("Unsupported concurrent container with id: {}. Supported containers are: \"SequentialConfig\", \"AssociativeConfig\"", id))),
        }
    }
}

impl BuildingBlockConfig for BuilderConfig {
    fn from_toml(value: toml::Value) -> Result<Self, ConfigError> {
        let table = match &value {
            toml::Value::Table(t) => t,
            _ => {
                return Err(ConfigError::ConfigFormatError(String::from(
                    "Building Block configuration must be a toml table.",
                )));
            }
        };

        let policy = match table.get("policy.kind").clone() {
            None => PolicyKind::None,
            Some(toml::value::Value::String(s)) => match s.as_ref() {
		"FIFO" => PolicyKind::FIFO,
		"LRU" => PolicyKind::LRU,
		"LRFU" => {
		    match table.get("policy.LRFU.exponent") {
			None => PolicyKind::LRFU(1.0),
			Some(&toml::value::Value::Float(f)) => PolicyKind::LRFU(f.clone() as f32),
			_ => return Err(ConfigError::ConfigFormatError(format!("Invalid exponent format for policy {},", s)))
		    }
		}
		_ => return Err(ConfigError::ConfigFormatError(format!("Invalid policy.kind value {}. Must be one of: FIFO, LRU, LRFU", s))),
            },
	    _ => return Err(ConfigError::ConfigFormatError(String::from("Invalid policy.kind type. Must be a string."))),
        };

        match GenericConfig::from_toml(value) {
            Ok(c) => Ok(BuilderConfig {
                config: c,
                policy: policy,
            }),
            Err(e) => Err(e),
        }
    }

    fn build<'a, K, V>(self) -> Box<dyn BuildingBlock<'a, K, V> + 'a>
    where
        K: 'a + GenericKey,
        V: 'a + GenericValue,
    {
        match self.policy {
            PolicyKind::None => self.config.build(),
            PolicyKind::FIFO => {
                Box::new(Policy::new(self.config.build(), FIFO::new()))
            }
            PolicyKind::LRU => Box::new(Policy::new(
                self.config.build(),
                LRU::<Counter>::new(),
            )),
            PolicyKind::LRFU(e) => Box::new(Policy::new(
                self.config.build(),
                LRFU::<Counter>::new(e),
            )),
        }
    }
}

/// Entry point to build a container from a generic configuration.
pub struct GenericConfig {
    id: String,
    toml_config: toml::Value,
}

impl GenericConfig {
    fn into_config<C: BuildingBlockConfig>(
        v: toml::Value,
    ) -> Result<C, ConfigError> {
        C::from_toml(v)
    }
}

impl BuildingBlockConfig for GenericConfig {
    /// Build a container from a toml value object representing a configuration.
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
        let ok = Ok(GenericConfig {
            id: id.clone(),
            toml_config: value.clone(),
        });

        // Check id field is a valid id and if it is, try to build the
        // associated config.

        match id.as_str() {
            "ArrayConfig" => {
                Self::into_config::<ArrayConfig>(value).and(ok)
            }
            "AssociativeConfig" => {
                Self::into_config::<AssociativeConfig>(value).and(ok)
            }
            "BatchConfig" => {
                Self::into_config::<BatchConfig>(value).and(ok)
            }
            "BTreeConfig" => {
                Self::into_config::<BTreeConfig>(value).and(ok)
            }
            #[cfg(feature = "compression")]
            "CompressorConfig" => {
                Self::into_config::<CompressorConfig>(value).and(ok)
            }
            "MultilevelConfig" => {
                Self::into_config::<MultilevelConfig>(value).and(ok)
            }
            "ProfilerConfig" => {
                Self::into_config::<ProfilerConfig>(value).and(ok)
            }
            "SequentialConfig" => {
                Self::into_config::<SequentialConfig>(value).and(ok)
            }
            #[cfg(feature = "stream")]
            "StreamConfig" => {
                Self::into_config::<StreamConfig>(value).and(ok)
            }
            unknown => Err(ConfigError::ConfigFormatError(format!(
                "Invalid container configuration type: {} 
Possible values are: {:?}.",
                unknown, CONFIGS
            ))),
        }
    }

    fn build<'a, K, V>(self) -> Box<dyn BuildingBlock<'a, K, V> + 'a>
    where
        K: 'a + GenericKey,
        V: 'a + GenericValue,
    {
        let table = self.toml_config.as_table().unwrap();
        let id = table.get("id").unwrap().as_str().unwrap();

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
            "CompressorConfig" => {
                Self::into_config::<CompressorConfig>(self.toml_config)
                    .unwrap()
                    .build()
            }
            "MultilevelConfig" => {
                Self::into_config::<MultilevelConfig>(self.toml_config)
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

#[cfg(test)]
mod tests {
    use super::BuilderConfig;
    use crate::config::{BuildingBlockConfig, ConfigError};
    use crate::BuildingBlock;

    #[test]
    fn test_generic_config() {
        let capacity = 10;
        let config_str =
            format!("id=\"ArrayConfig\"\ncapacity={}", capacity);
        let array: Box<dyn BuildingBlock<u64, u64>> =
            BuilderConfig::from_str(config_str.as_str())
                .unwrap()
                .build();
        assert_eq!(array.capacity(), capacity);
    }

    #[test]
    fn test_invalid_id_config() {
        let config_str = format!("id=\"Array\"\ncapacity=10");
        assert!(matches!(
            BuilderConfig::from_str(config_str.as_str()),
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
policy='FIFO'
",
            capacity
        );
        let array: Box<dyn BuildingBlock<u64, u64>> =
            BuilderConfig::from_str(config_str.as_str())
                .unwrap()
                .build();
        assert_eq!(array.capacity(), capacity);
    }
    #[test]
    fn test_lrfu_config() {
        let capacity = 10;
        let config_str = format!(
            "
id='ArrayConfig'
capacity={}
policy.kind='LRFU'
policy.LRFU.exponent=0.5
",
            capacity
        );
        let array: Box<dyn BuildingBlock<u64, u64>> =
            BuilderConfig::from_str(config_str.as_str())
                .unwrap()
                .build();
        assert_eq!(array.capacity(), capacity);
    }
}
