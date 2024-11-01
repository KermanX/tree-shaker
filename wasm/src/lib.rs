use oxc::{
  allocator::Allocator, codegen::CodegenOptions, minifier::MinifierOptions, span::SourceType,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(getter_with_clone)]
pub struct Result {
  pub output: String,
  pub diagnostics: Vec<String>,
  pub logs: Vec<String>,
}

#[wasm_bindgen]
pub fn tree_shake(input: String, do_tree_shake: bool, do_minify: bool, logging: bool) -> Result {
  let result = tree_shake::tree_shake(tree_shake::TreeShakeOptions {
    config: tree_shake::TreeShakeConfig::default(),
    allocator: &Allocator::default(),
    source_type: SourceType::default(),
    source_text: input,
    tree_shake: do_tree_shake,
    minify_options: do_minify.then(|| MinifierOptions::default()),
    codegen_options: CodegenOptions { minify: do_minify, ..Default::default() },
    logging,
  });
  Result {
    output: result.codegen_return.code,
    diagnostics: result.diagnostics.into_iter().collect(),
    logs: result.logs,
  }
}
