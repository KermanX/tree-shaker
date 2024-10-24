#![deny(clippy::all)]

use oxc::{
  allocator::Allocator, codegen::CodegenOptions, minifier::MinifierOptions, span::SourceType,
};

#[macro_use]
extern crate napi_derive;

#[napi]
pub struct TreeShakeResultBinding {
  pub output: String,
  pub diagnostics: Vec<String>,
}

#[napi]
pub fn tree_shake(
  input: String,
  tree_shake: Option<String>,
  do_minify: bool,
) -> TreeShakeResultBinding {
  let result = tree_shake::tree_shake(tree_shake::TreeShakeOptions {
    config: match tree_shake.as_deref() {
      Some("safest") => tree_shake::TreeShakeConfig::safest(),
      Some("recommended") => tree_shake::TreeShakeConfig::recommended(),
      Some("smallest") => tree_shake::TreeShakeConfig::smallest(),
      None => tree_shake::TreeShakeConfig::default(),
      _ => panic!("Invalid tree shake option"),
    },
    allocator: &Allocator::default(),
    source_type: SourceType::default(),
    source_text: input,
    tree_shake: tree_shake.is_some(),
    minify: do_minify.then(MinifierOptions::default),
    code_gen: CodegenOptions { minify: do_minify, ..Default::default() },
    logging: true,
  });
  TreeShakeResultBinding {
    output: result.codegen_return.code,
    diagnostics: result.diagnostics.into_iter().collect(),
  }
}
