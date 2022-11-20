use crate::builder::{Build, PolicyBuilder};
use crate::config::{
    ConfigError, ConfigInstance, ConfigWithTraits, DynOrdered,
    GenericConfig, GenericKey, GenericValue, IntoConfig,
};
use crate::policy::{
    timestamp::{Counter, Timestamp},
    Fifo, Lrfu, Lru,
};
use crate::{BuildingBlock, Policy};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Copy, Clone)]
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
/// use byoc::config::{ConfigBuilder, DynBuildingBlock};
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
///                ConfigBuilder::from_string(config_str.as_str())
///                .unwrap()
///                .build();
/// ```
#[derive(Deserialize, Serialize, Clone)]
pub struct PolicyConfig {
    #[allow(dead_code)]
    id: String,
    policy: PolicyKind,
    container: toml::Value,
}

impl PolicyConfig {
    fn from_builder<C: ConfigInstance, B: IntoConfig<C>>(
        builder: &B,
        policy: PolicyKind,
    ) -> Self {
        let container_config_str = builder.into_config().to_toml_string();
        let container: toml::value::Value =
            toml::de::from_str(container_config_str.as_ref()).unwrap();

        PolicyConfig {
            id: String::from(PolicyConfig::id()),
            policy,
            container,
        }
    }
}

impl<C, V, B, T> IntoConfig<PolicyConfig>
    for PolicyBuilder<C, V, Lru<T>, B>
where
    C: ConfigInstance,
    B: IntoConfig<C>,
    T: Timestamp,
{
    fn into_config(&self) -> PolicyConfig {
        PolicyConfig::from_builder(&self.builder, PolicyKind::Lru)
    }
}

impl<C, V, B, T> IntoConfig<PolicyConfig>
    for PolicyBuilder<C, V, Lrfu<T>, B>
where
    C: ConfigInstance,
    B: IntoConfig<C>,
    T: Timestamp,
{
    fn into_config(&self) -> PolicyConfig {
        PolicyConfig::from_builder(
            &self.builder,
            PolicyKind::Lrfu(self.policy.exponent()),
        )
    }
}

impl<C, V, B> IntoConfig<PolicyConfig> for PolicyBuilder<C, V, Fifo, B>
where
    C: ConfigInstance,
    B: IntoConfig<C>,
{
    fn into_config(&self) -> PolicyConfig {
        PolicyConfig::from_builder(&self.builder, PolicyKind::Fifo)
    }
}

impl ConfigInstance for PolicyConfig {
    fn id() -> &'static str {
        "PolicyConfig"
    }

    fn from_toml(value: &toml::Value) -> Result<Self, ConfigError> {
        let toml = toml::to_string(&value).unwrap();
        toml::from_str(&toml).map_err(ConfigError::TomlFormatError)
    }
}

impl<'a, K, V> Build<Box<dyn BuildingBlock<'a, K, V> + 'a>>
    for PolicyConfig
where
    K: 'a + GenericKey,
    V: 'a + GenericValue,
{
    fn build(self) -> Box<dyn BuildingBlock<'a, K, V> + 'a> {
        match self.policy {
            PolicyKind::Lrfu(exponent) => Box::new(Policy::new(
                DynOrdered::new(
                    GenericConfig::from_toml(&self.container)
                        .unwrap()
                        .build(),
                    false,
                ),
                Lrfu::<Counter>::new(exponent),
            )),
            PolicyKind::Lru => Box::new(Policy::new(
                DynOrdered::new(
                    GenericConfig::from_toml(&self.container)
                        .unwrap()
                        .build(),
                    false,
                ),
                Lru::<Counter>::new(),
            )),
            PolicyKind::Fifo => Box::new(Policy::new(
                DynOrdered::new(
                    GenericConfig::from_toml(&self.container)
                        .unwrap()
                        .build(),
                    false,
                ),
                Fifo::new(),
            )),
            PolicyKind::None => {
                GenericConfig::from_toml(&self.container).unwrap().build()
            }
        }
    }
}

impl ConfigWithTraits for PolicyConfig {}

#[cfg(test)]
mod tests {
    use super::PolicyConfig;
    use crate::builder::{ArrayBuilder, Build, PolicyBuilder};
    use crate::config::tests::test_config_builder;
    use crate::config::{ConfigError, ConfigInstance};
    use crate::utils::policy::Fifo;
    use crate::BuildingBlock;

    #[test]
    fn test_valid_policy_config() {
        let array_capacity = 10;
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
        let config = PolicyConfig::from_toml(&value).unwrap();
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
            PolicyConfig::from_toml(&value),
            Err(ConfigError::TomlFormatError(_))
        ));
    }

    #[test]
    fn test_builder_into_config() {
        let builder = PolicyBuilder::<_, (), _, _>::new(
            ArrayBuilder::<()>::new(2),
            Fifo::new(),
        );
        test_config_builder(builder);
    }
}
