use crate::BuildingBlock;
use serde::de::DeserializeOwned;
use toml;

pub trait BuildingBlockConfig<'a, K, V>: DeserializeOwned + Clone
where
    K: 'a,
    V: 'a,
{
    fn build(self) -> Box<dyn BuildingBlock<'a, K, V> + 'a>;
    fn from_toml(v: toml::Value) -> Result<Self, toml::de::Error> {
        let toml = toml::to_string(&v).unwrap();
        toml::from_str(&toml)
    }
}

mod config;
pub use config::Builder;

mod array;
pub use array::ArrayConfig;

mod associative;
pub use associative::AssociativeConfig;

mod batch;
pub use batch::BatchConfig;

mod btree;
pub use btree::BTreeConfig;

mod compression;
pub use compression::CompressorConfig;

mod multilevel;
pub use multilevel::MultilevelConfig;

mod policy;
pub use policy::PolicyConfig;

mod profiler;
pub use profiler::ProfilerConfig;

mod sequential;
pub use sequential::SequentialConfig;

mod stream;
pub use stream::StreamConfig;
