use crate::config::{ConfigInstance, IntoConfig};

pub(crate) fn test_config_builder<B, C>(builder: B)
where
    B: IntoConfig<C>,
    C: ConfigInstance,
{
    // Building configuration object succeeds or panics.
    let config: C = builder.into_config();
    // Building container from configuration object succeeds or panics.
    println!(
        "{} into {}: \n{}",
        std::any::type_name::<B>(),
        std::any::type_name::<C>(),
        config.to_toml_string_pretty()
    );
}
