use swc_core::ecma::{
    ast::*,
    visit::{as_folder, FoldWith},
};
use swc_core::plugin::{plugin_transform, proxies::TransformPluginProgramMetadata};
use transform::TransformVisitor;

mod transform;

#[plugin_transform]
pub fn process_transform(program: Program, data: TransformPluginProgramMetadata) -> Program {
    let remove_imports_transform: TransformVisitor = serde_json::from_str(
        &data
            .get_transform_plugin_config()
            .expect("failed to get plugin config"),
    )
    .expect("invalid config");

    program.fold_with(&mut as_folder(remove_imports_transform))
}
