use crate::config::{
    BuildingBlockConfig, ConfigError, GenericConfig, GenericKey,
    GenericValue,
};
use crate::policies::{timestamp::Counter, FIFO, LRFU, LRU};
use crate::{BuildingBlock, Policy};
use serde::Deserialize;
use toml;

#[derive(Deserialize, Clone)]
#[serde(tag = "kind", content = "exponent")]
pub enum PolicyKind {
    LRFU(f32),
    LRU,
    FIFO,
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
/// [`LRFU`](../policies/struct.LRFU.html) policy.
///
/// Below is an example of the configuration of a
/// [`Policy`](../struct.Policy.html) wrapping an
/// [`Array`](../struct.Array.html) container.
/// ```no_run
/// use byoc::BuildingBlock;
/// use byoc::builder::traits::Builder;
/// use byoc::config::{BuilderConfig, BuildingBlockConfig};
///
/// let config_str = format!("
/// id='PolicyConfig'
/// policy.kind='FIFO'
/// [container]
/// id='ArrayConfig'
/// capacity=10
/// ");
///
/// let container: Box<dyn BuildingBlock<u64, u64>> =
///                BuilderConfig::from_str(config_str.as_str())
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
            PolicyKind::LRFU(exponent) => Box::new(Policy::new(
                GenericConfig::from_toml(self.container).unwrap().build(),
                LRFU::<Counter>::new(exponent),
            )),
            PolicyKind::LRU => Box::new(Policy::new(
                GenericConfig::from_toml(self.container).unwrap().build(),
                LRU::<Counter>::new(),
            )),
            PolicyKind::FIFO => Box::new(Policy::new(
                GenericConfig::from_toml(self.container).unwrap().build(),
                FIFO::new(),
            )),
            PolicyKind::None => {
                GenericConfig::from_toml(self.container).unwrap().build()
            }
        }
    }

    fn from_toml(value: toml::Value) -> Result<Self, ConfigError> {
        let toml = toml::to_string(&value).unwrap();
        match toml::from_str(&toml) {
            Err(e) => Err(ConfigError::TomlFormatError(e)),
            Ok(cfg) => Ok(cfg),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::{BuildingBlockConfig, ConfigError, PolicyConfig};
    use crate::BuildingBlock;
    use toml;

    #[test]
    fn test_valid_policy_config() {
        let array_capacity = 10;
        let config_str = format!(
            "
id='PolicyConfig'
policy.kind='LRFU'
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
        let config_str = format!(
            "
id='PolicyConfig'
policy.kind='LRF'
policy.exponent=0.5
[container]
id='ArrayConfig'
capacity=10
"
        );
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        assert!(matches!(
            PolicyConfig::from_toml(value),
            Err(ConfigError::TomlFormatError(_))
        ));
    }
}
