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
  do_tree_shake: bool,
  do_minify: bool,
  eval_mode: bool,
) -> TreeShakeResultBinding {
  let result = tree_shake::tree_shake(tree_shake::TreeShakeOptions {
    config: tree_shake::TreeShakeConfig::default(),
    allocator: &Allocator::default(),
    source_type: SourceType::default(),
    source_text: input,
    tree_shake: do_tree_shake,
    minify: do_minify.then(|| MinifierOptions::default()),
    code_gen: CodegenOptions { single_quote: true, minify: true },
    eval_mode,
  });
  TreeShakeResultBinding {
    output: result.codegen_return.source_text,
    diagnostics: result.diagnostics.iter().map(|d| d.to_string()).collect(),
  }
}
