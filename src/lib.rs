mod config;
mod effector;
mod macros;
mod path;
mod sid;

use swc_core::{
    ecma::{ast::Program, visit::VisitMutWith},
    plugin::{
        metadata::TransformPluginMetadataContextKind, plugin_transform,
        proxies::TransformPluginProgramMetadata,
    },
};

pub use crate::{
    config::{Config, InternalConfig, PublicConfig},
    effector::Effector,
};

#[plugin_transform]
pub fn effector(mut program: Program, data: TransformPluginProgramMetadata) -> Program {
    // TODO: Correct errors
    let public_config = serde_json::from_str::<PublicConfig>(
        &data.get_transform_plugin_config().expect("Failed to get plugin config"),
    )
    .expect("Invalid config!");

    let no_defaults = public_config.no_defaults;

    let root = data.get_context(&TransformPluginMetadataContextKind::Cwd);

    let filename = data.get_context(&TransformPluginMetadataContextKind::Filename);

    let config = Config::new(public_config, InternalConfig::new(no_defaults));

    let mut plugin = Effector::new(config, root.as_deref(), filename.as_deref(), data.source_map);

    program.visit_mut_with(&mut plugin);

    program
}
