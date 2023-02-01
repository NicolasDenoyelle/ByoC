use crate::builder::ProfilerBuilder;
use crate::config::{
    ConfigError, ConfigInstance, GenericConfig, GenericKey, GenericValue,
    IntoConfig,
};
use crate::objsafe::DynBuildingBlock;
use crate::utils::profiler::ProfilerOutputKind;
use crate::Profiler;
use serde::{Deserialize, Serialize};

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
/// use byoc::{BuildingBlock, DynBuildingBlock};
/// use byoc::config::{ConfigInstance, ConfigBuilder};
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
/// let container: DynBuildingBlock<u64, u64> =
///                ConfigBuilder::from_string(config_str.as_str())
///                .unwrap()
///                .build();
/// ```
#[derive(Deserialize, Serialize, Clone)]
pub struct ProfilerConfig {
    #[allow(dead_code)]
    id: String,
    name: String,
    output: ProfilerOutputKind,
    container: toml::Value,
}

impl<C, B> IntoConfig<ProfilerConfig> for ProfilerBuilder<C, B>
where
    C: ConfigInstance,
    B: IntoConfig<C>,
{
    fn as_config(&self) -> ProfilerConfig {
        let container_toml_str = self.builder.as_config().to_toml_string();
        let container: toml::value::Value =
            toml::de::from_str(container_toml_str.as_ref()).unwrap();
        ProfilerConfig {
            id: String::from(ProfilerConfig::id()),
            name: self.name.clone(),
            output: self.output.clone(),
            container,
        }
    }
}

impl ConfigInstance for ProfilerConfig {
    fn id() -> &'static str {
        "ProfilerConfig"
    }

    fn from_toml(value: &toml::Value) -> Result<Self, ConfigError> {
        let toml = toml::to_string(&value).unwrap();
        let cfg: ProfilerConfig = match toml::from_str(&toml) {
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
        let is_concurrent = self.is_concurrent();
        DynBuildingBlock::new(
            Profiler::new(
                &self.name,
                self.output,
                GenericConfig::from_toml(&self.container).unwrap().build(),
            ),
            is_concurrent,
        )
    }

    fn is_concurrent(&self) -> bool {
        GenericConfig::from_toml(&self.container)
            .unwrap()
            .is_concurrent()
    }
}

#[cfg(test)]
mod tests {
    use super::ProfilerConfig;
    use crate::builder::{ArrayBuilder, ProfilerBuilder};
    use crate::config::tests::test_config_builder;
    use crate::config::{ConfigError, ConfigInstance};
    use crate::objsafe::DynBuildingBlock;
    use crate::BuildingBlock;

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
        let config = ProfilerConfig::from_toml(&value).unwrap();
        let container: DynBuildingBlock<u64, u64> = config.build();
        assert_eq!(container.capacity(), array_capacity);
    }

    #[test]
    fn test_invalid_profiler_config() {
        let config_str = "
id='ProfilerConfig'
name=10
output.kind='None'
[container]
id='ArrayConfig'
capacity=10
"
        .to_string();
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        assert!(matches!(
            ProfilerConfig::from_toml(&value),
            Err(ConfigError::TomlFormatError(_))
        ));
    }

    #[test]
    fn test_builder_as_config() {
        let builder = ProfilerBuilder::new(
            "config_builder_test",
            crate::utils::profiler::ProfilerOutputKind::None,
            ArrayBuilder::<()>::new(2),
        );
        test_config_builder(builder);
    }
}
