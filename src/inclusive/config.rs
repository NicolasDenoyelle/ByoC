use crate::builder::{Build, InclusiveBuilder};
use crate::config::{
    ConfigError, ConfigInstance, ConfigWithTraits, GenericConfig,
    GenericKey, GenericValue, IntoConfig,
};
use crate::{BuildingBlock, Inclusive};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct InclusiveConfig {
    #[allow(dead_code)]
    id: String,
    front: toml::Value,
    back: toml::Value,
}

impl<L, LB, R, RB> IntoConfig<InclusiveConfig>
    for InclusiveBuilder<L, LB, R, RB>
where
    LB: IntoConfig<L>,
    RB: IntoConfig<R>,
    L: ConfigInstance,
    R: ConfigInstance,
{
    fn into_config(&self) -> InclusiveConfig {
        let left_config: L = self.lbuilder.into_config();
        let right_config: R = self.rbuilder.into_config();
        let left_config_str = left_config.to_toml_string();
        let right_config_str = right_config.to_toml_string();
        let front = toml::de::from_str(left_config_str.as_ref()).unwrap();
        let back = toml::de::from_str(right_config_str.as_ref()).unwrap();

        InclusiveConfig {
            id: String::from(InclusiveConfig::id()),
            front,
            back,
        }
    }
}

impl ConfigInstance for InclusiveConfig {
    fn id() -> &'static str {
        "InclusiveConfig"
    }

    fn from_toml(value: &toml::Value) -> Result<Self, ConfigError> {
        let toml = toml::to_string(&value).unwrap();
        let cfg: InclusiveConfig = match toml::from_str(&toml) {
            Err(e) => return Err(ConfigError::TomlFormatError(e)),
            Ok(cfg) => cfg,
        };
        match (
            GenericConfig::from_toml(&cfg.front),
            GenericConfig::from_toml(&cfg.back),
        ) {
            (Ok(_), Ok(_)) => Ok(cfg),
            (Ok(_), Err(e)) => Err(e),
            (Err(e), _) => Err(e),
        }
    }
}

impl<'a, K, V> Build<Box<dyn BuildingBlock<'a, K, V> + 'a>>
    for InclusiveConfig
where
    K: 'a + GenericKey,
    V: 'a + GenericValue,
{
    fn build(self) -> Box<dyn BuildingBlock<'a, K, V> + 'a> {
        Box::new(Inclusive::new(
            GenericConfig::from_toml(&self.front).unwrap().build(),
            GenericConfig::from_toml(&self.back).unwrap().build(),
        ))
    }
}

impl ConfigWithTraits for InclusiveConfig {
    fn is_ordered(&self) -> bool {
        GenericConfig::from_toml(&self.front)
            .unwrap()
            .has_ordered_trait
            && GenericConfig::from_toml(&self.back)
                .unwrap()
                .has_ordered_trait
    }
}

#[cfg(test)]
mod tests {
    use super::InclusiveConfig;
    use crate::builder::{ArrayBuilder, Build, InclusiveBuilder};
    use crate::config::tests::test_config_builder;
    use crate::config::{ConfigError, ConfigInstance};
    use crate::BuildingBlock;

    #[test]
    fn test_valid_inclusive_config() {
        let array_capacity = 10;
        let config_str = format!(
            "id='InclusiveConfig'
[front]
id='ArrayConfig'
capacity={}
[back]
id='ArrayConfig'
capacity={}
",
            array_capacity, array_capacity
        );
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        let config = InclusiveConfig::from_toml(&value).unwrap();
        let container: Box<dyn BuildingBlock<u64, u64>> = config.build();
        assert_eq!(container.capacity(), array_capacity * 2);
    }

    #[test]
    fn test_invalid_inclusive_config() {
        let config_str = "id='InclusiveConfig'
[front]
id='ArrayConfig'
capacity=10
[back]
id='ArrayConfig'
toto='titi'
"
        .to_string();
        let value: toml::Value =
            toml::from_str(config_str.as_str()).unwrap();
        assert!(matches!(
            InclusiveConfig::from_toml(&value),
            Err(ConfigError::ConfigFormatError(_))
        ));
    }

    #[test]
    fn test_builder_into_config() {
        let builder = InclusiveBuilder::new(
            ArrayBuilder::<()>::new(2),
            ArrayBuilder::<()>::new(2),
        );
        test_config_builder(builder);
    }
}
