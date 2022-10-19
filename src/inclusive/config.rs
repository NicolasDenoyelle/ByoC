use crate::config::{
    BuildingBlockConfig, ConfigError, GenericConfig, GenericKey,
    GenericValue,
};
use crate::{BuildingBlock, Inclusive};
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct InclusiveConfig {
    #[allow(dead_code)]
    id: String,
    front: toml::Value,
    back: toml::Value,
}

impl BuildingBlockConfig for InclusiveConfig {
    fn build<'a, K, V>(self) -> Box<dyn BuildingBlock<'a, K, V> + 'a>
    where
        K: 'a + GenericKey,
        V: 'a + GenericValue,
    {
        Box::new(Inclusive::new(
            GenericConfig::from_toml(self.front).unwrap().build(),
            GenericConfig::from_toml(self.back).unwrap().build(),
        ))
    }

    fn is_ordered(&self) -> bool {
        GenericConfig::from_toml(self.front.clone())
            .unwrap()
            .has_ordered_trait
            && GenericConfig::from_toml(self.back.clone())
                .unwrap()
                .has_ordered_trait
    }

    fn from_toml(value: toml::Value) -> Result<Self, ConfigError> {
        let toml = toml::to_string(&value).unwrap();
        let cfg: InclusiveConfig = match toml::from_str(&toml) {
            Err(e) => return Err(ConfigError::TomlFormatError(e)),
            Ok(cfg) => cfg,
        };
        match (
            GenericConfig::from_toml(cfg.front.clone()),
            GenericConfig::from_toml(cfg.back.clone()),
        ) {
            (Ok(_), Ok(_)) => Ok(cfg),
            (Ok(_), Err(e)) => Err(e),
            (Err(e), _) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::InclusiveConfig;
    use crate::config::{BuildingBlockConfig, ConfigError};
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
        let config = InclusiveConfig::from_toml(value).unwrap();
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
            InclusiveConfig::from_toml(value),
            Err(ConfigError::ConfigFormatError(_))
        ));
    }
}
