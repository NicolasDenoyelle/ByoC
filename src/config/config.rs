use crate::config::{
    ArrayConfig,
    AssociativeConfig,
    BTreeConfig,
    BatchConfig,
    BuildingBlockConfig,
    CompressorConfig,
    MultilevelConfig,
    // PolicyConfig,
    ProfilerConfig,
    SequentialConfig,
    StreamConfig,
};
use crate::BuildingBlock;

use serde::{de::DeserializeOwned, Serialize};
use std::cmp::Ord;
use std::hash::Hash;
use std::io::Read;
use toml;

pub trait GenericKey:
    Ord + Copy + Hash + Serialize + DeserializeOwned
{
}
impl<T: Ord + Copy + Hash + Serialize + DeserializeOwned> GenericKey
    for T
{
}

pub trait GenericValue: Ord + Serialize + DeserializeOwned {}
impl<T: Ord + Serialize + DeserializeOwned> GenericValue for T {}

pub struct Builder<'a, K, V>
where
    K: 'a + GenericKey,
    V: 'a + GenericValue,
{
    _unused: std::marker::PhantomData<&'a (K, V)>,
}

pub type DynBuildingBlock<'a, K, V> =
    Box<dyn BuildingBlock<'a, K, V> + 'a>;

impl<'a, K, V> Builder<'a, K, V>
where
    K: 'a + GenericKey,
    V: 'a + GenericValue,
{
    pub fn from_str(
        s: &str,
    ) -> Result<DynBuildingBlock<'a, K, V>, toml::de::Error> {
        let value = toml::from_str::<toml::Value>(s)?;
        Ok(Self::from_toml(value))
    }

    pub fn from_file<P: AsRef<std::path::Path> + std::fmt::Debug>(
        path: P,
    ) -> Result<DynBuildingBlock<'a, K, V>, toml::de::Error> {
        let mut file = std::fs::File::open(&path)
            .expect(format!("Invalid file path: {:?}", path).as_str());
        let mut s = String::from("");
        file.read_to_string(&mut s)
            .expect(format!("Error reading: {:?}", path).as_str());
        Self::from_str(s.as_str())
    }

    fn into_config<C: BuildingBlockConfig<'a, K, V>>(
        value: toml::Value,
    ) -> C {
        match C::from_toml(value) {
            Err(e) => {
                panic!(
                    "Invalid {} format: {}",
                    std::any::type_name::<C>(),
                    e
                )
            }
            Ok(c) => c,
        }
    }

    pub fn from_toml(value: toml::Value) -> DynBuildingBlock<'a, K, V>
    where
        K: 'a + GenericKey,
        V: 'a + GenericValue,
    {
        let table = match value {
            toml::Value::Table(t) => t,
            _ => panic!(
                "Building Block configuration must be a toml table."
            ),
        };
        let id = String::from(table.get("id")
            .expect("All configurations must have an 'id' field matching the config type.").as_str().expect("'id' associated value must be a string."));
        let value = toml::Value::Table(table);
        match id.as_str() {
            "ArrayConfig" => Self::into_config::<ArrayConfig>(value).build(),
            "AssociativeConfig" => Self::into_config::<AssociativeConfig>(value).build(),
            "BatchConfig" => Self::into_config::<BatchConfig>(value).build(),
            "BTreeConfig" => Self::into_config::<BTreeConfig>(value).build(),
            "CompressorConfig" => Self::into_config::<CompressorConfig>(value).build(),
            "MultilevelConfig" => Self::into_config::<MultilevelConfig>(value).build(),
            // "PolicyConfig" => Self::into_config::<PolicyConfig>(value).build(),
            "ProfilerConfig" => Self::into_config::<ProfilerConfig>(value).build(),
            "SequentialConfig" => Self::into_config::<SequentialConfig>(value).build(),
            "StreamConfig" => Self::into_config::<StreamConfig>(value).build(),
            _ => panic!("Invalid container configuration type: {}\n Possible values are: ArrayConfig, AssociativeConfig, BatchConfig, BTreeConfig, MultilevelConfig.", id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Builder;
    use crate::BuildingBlock;

    #[test]
    fn test_generic_config() {
        let capacity = 10;
        let config_str =
            format!("id=\"ArrayConfig\"\ncapacity={}", capacity);
        let array: Box<dyn BuildingBlock<u64, u64>> =
            Builder::from_str(config_str.as_str()).unwrap();
        assert_eq!(array.capacity(), capacity);
    }
}
