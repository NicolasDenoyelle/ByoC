use crate::config::{
    BuildingBlockConfig, ConfigError, DynOrdered, GenericConfig,
    GenericKey, GenericValue,
};
use crate::policy::{timestamp::Counter, Fifo, Lrfu, Lru};
use crate::{BuildingBlock, Policy};
use serde::Deserialize;
use toml;

#[derive(Deserialize, Copy, Clone)]
#[serde(tag = "kind", content = "exponent")]
pub enum PolicyKind {
    Lrfu(f32),
    Lru,
    Fifo,
    None,
}

/// Configuration format for [`Policy`](../struct.Policy.html)
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
/// * an `id` field where the `id` value must be "PolicyConfig",
/// * `policy.kind` field which accept values defined in the [`PolicyKind`]
/// enum,
/// * `policy.exponent` field that sets the floating point value for the
/// [`Lrfu`](../policy/struct.Lrfu.html) policy.
///
/// Below is an example of the configuration of a
/// [`Policy`](../struct.Policy.html) wrapping an
/// [`Array`](../struct.Array.html) container.
/// ```no_run
/// use byoc::BuildingBlock;
/// use byoc::builder::Build;
/// use byoc::config::{Builder, DynBuildingBlock};
///
/// let config_str = format!("
/// id='PolicyConfig'
/// policy.kind='Fifo'
/// [container]
/// id='ArrayConfig'
/// capacity=10
/// ");
///
/// let container: DynBuildingBlock<u64, u64> =
///                Builder::from_string(config_str.as_str())
///                .unwrap()
///                .build();
/// ```
#[derive(Deserialize, Clone)]
pub struct PolicyConfig {
    #[allow(dead_code)]
    id: String,
    policy: PolicyKind,
    container: toml::Value,
}

impl BuildingBlockConfig for PolicyConfig {
    fn build<'a, K, V>(self) -> Box<dyn BuildingBlock<'a, K, V> + 'a>
    where
        K: 'a + GenericKey,
        V: 'a + GenericValue,
    {
        match self.policy {
            PolicyKind::Lrfu(exponent) => Box::new(Policy::new(
                DynOrdered::new(
                    GenericConfig::from_toml(self.container)
                        .unwrap()
                        .build(),
                    false,
                ),
                Lrfu::<Counter>::new(exponent),
            )),
            PolicyKind::Lru => Box::new(Policy::new(
                DynOrdered::new(
                    GenericConfig::from_toml(self.container)
                        .unwrap()
                        .build(),
                    false,
                ),
                Lru::<Counter>::new(),
            )),
            PolicyKind::Fifo => Box::new(Policy::new(
                DynOrdered::new(
                    GenericConfig::from_toml(self.container)
                        .unwrap()
                        .build(),
                    false,
                ),
                Fifo::new(),
            )),
            PolicyKind::None => {
                GenericConfig::from_toml(self.container).unwrap().build()
            }
        }
    }

    fn from_toml(value: toml::Value) -> Result<Self, ConfigError> {
        let toml = toml::to_string(&value).unwrap();
        toml::from_str(&toml).map_err(ConfigError::TomlFormatError)
    }
}

#[cfg(test)]
mod tests {
    use super::PolicyConfig;
    use crate::config::{BuildingBlockConfig, ConfigError};
    use crate::{Array, BuildingBlock};
    use toml;

    #[test]
    fn test_valid_policy_config() {
        let array_capacity = Array::<(u64, u64)>::element_size() * 10;
        let config_str = format!(
            "
id='PolicyConfig'
policy.kind='Lrfu'
policy.exponent=0.5
[container]
id='ArrayConfig'
capacity={}
",
            array_capacity
        );
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        let config = PolicyConfig::from_toml(value).unwrap();
        let container: Box<dyn BuildingBlock<u64, u64>> = config.build();
        assert_eq!(container.capacity(), array_capacity);
    }

    #[test]
    fn test_invalid_policy_config() {
        let config_str = "
id='PolicyConfig'
policy.kind='LRF'
policy.exponent=0.5
[container]
id='ArrayConfig'
capacity=10
"
        .to_string();
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        assert!(matches!(
            PolicyConfig::from_toml(value),
            Err(ConfigError::TomlFormatError(_))
        ));
    }
}
