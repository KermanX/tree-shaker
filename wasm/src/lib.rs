use oxc::{codegen::CodegenOptions, minifier::MinifierOptions, span::SourceType};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(getter_with_clone)]
pub struct Result {
  pub output: String,
  pub diagnostics: Vec<String>,
  pub logs: Vec<String>,
}

#[wasm_bindgen]
pub fn tree_shake(
  source_text: String,
  do_tree_shake: bool,
  do_minify: bool,
  logging: bool,
) -> Result {
  let result = tree_shake::tree_shake(
    source_text,
    tree_shake::TreeShakeOptions {
      config: if do_tree_shake {
        tree_shake::TreeShakeConfig::recommended()
      } else {
        tree_shake::TreeShakeConfig::disabled()
      }.with_react_jsx(true),
      minify_options: do_minify.then(|| MinifierOptions::default()),
      codegen_options: CodegenOptions { minify: do_minify, ..Default::default() },
      logging,
    },
  );
  Result {
    output: result.codegen_return.code,
    diagnostics: result.diagnostics.into_iter().collect(),
    logs: result.logs,
  }
}
