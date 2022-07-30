use crate::config::{
    BuildingBlockConfig, ConfigError, GenericConfig, GenericKey,
    GenericValue,
};
use crate::{BuildingBlock, Profiler, ProfilerOutputKind};
use serde::Deserialize;
use toml;

/// Configuration format for [`Profiler`](../struct.Profiler.html)
/// containers.
///
/// This configuration format is composed of an `id` field where the
/// `id` value must be "ProfilerConfig", `name` as the profiler name,
/// `output` as the destination where the profile information will be written
/// and the configuration in toml format of the container to profile.
///
/// Below is an example of the configuration of a
/// [`Profiler`](../struct.Profiler.html) wrapping an
/// [`Array`](../struct.Array.html) container and writing the profiler
/// information to stdout.
/// ```
/// use byoc::BuildingBlock;
/// use byoc::builder::traits::Builder;
/// use byoc::config::{BuilderConfig, BuildingBlockConfig};
///
/// let config_str = format!("
/// id='ProfilerConfig'
/// name='test_profiler'
/// output.kind='Stdout'
/// [container]
/// id='ArrayConfig'
/// capacity=10
/// ");
///
/// // "output" fields could have also been:
/// // output.kind='File'
/// // output.filename='/dev/stdout'
///
/// let container: Box<dyn BuildingBlock<u64, u64>> =
///                BuilderConfig::from_str(config_str.as_str())
///                .unwrap()
///                .build();
/// ```
#[derive(Deserialize, Clone)]
pub struct ProfilerConfig {
    #[allow(dead_code)]
    id: String,
    name: String,
    output: ProfilerOutputKind,
    container: toml::Value,
}

impl BuildingBlockConfig for ProfilerConfig {
    fn build<'a, K, V>(self) -> Box<dyn BuildingBlock<'a, K, V> + 'a>
    where
        K: 'a + GenericKey,
        V: 'a + GenericValue,
    {
        Box::new(Profiler::new(
            &self.name,
            self.output,
            GenericConfig::from_toml(self.container).unwrap().build(),
        ))
    }

    fn from_toml(value: toml::Value) -> Result<Self, ConfigError> {
        let toml = toml::to_string(&value).unwrap();
        let cfg: ProfilerConfig = match toml::from_str(&toml) {
            Err(e) => return Err(ConfigError::TomlFormatError(e)),
            Ok(cfg) => cfg,
        };
        match GenericConfig::from_toml(cfg.container.clone()) {
            Ok(_) => Ok(cfg),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::{
        BuildingBlockConfig, ConfigError, ProfilerConfig,
    };
    use crate::BuildingBlock;
    use toml;

    #[test]
    fn test_valid_profiler_config() {
        let array_capacity = 10;
        let config_str = format!(
            "
id='ProfilerConfig'
name='test_profiler'
output.kind='None'
[container]
id='ArrayConfig'
capacity={}
",
            array_capacity
        );
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        let config = ProfilerConfig::from_toml(value).unwrap();
        let container: Box<dyn BuildingBlock<u64, u64>> = config.build();
        assert_eq!(container.capacity(), array_capacity);
    }

    #[test]
    fn test_invalid_profiler_config() {
        let config_str = format!(
            "
id='ProfilerConfig'
name=10
output.kind='None'
[container]
id='ArrayConfig'
capacity=10
"
        );
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        assert!(matches!(
            ProfilerConfig::from_toml(value),
            Err(ConfigError::TomlFormatError(_))
        ));
    }
}
