extern crate console_error_panic_hook;

use oxc::{
  codegen::CodegenOptions,
  minifier::{MangleOptions, MinifierOptions},
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(getter_with_clone)]
pub struct Result {
  pub output: String,
  pub diagnostics: Vec<String>,
}

#[wasm_bindgen]
pub fn tree_shake(
  source_text: String,
  preset: String,
  do_minify: bool,
  always_inline_literal: bool,
) -> Result {
  console_error_panic_hook::set_once();

  let result = tree_shake::tree_shake(
    source_text,
    tree_shake::TreeShakeOptions {
      config: match preset.as_str() {
        "recommended" => tree_shake::TreeShakeConfig::recommended(),
        "smallest" => tree_shake::TreeShakeConfig::smallest(),
        "safest" => tree_shake::TreeShakeConfig::safest(),
        "disabled" => tree_shake::TreeShakeConfig::disabled(),
        _ => unreachable!("Invalid preset {}", preset),
      }
      .with_react_jsx(true)
      .with_always_inline_literal(always_inline_literal),
      minify_options: do_minify.then_some({
        MinifierOptions {
          mangle: Some(MangleOptions { top_level: true, ..Default::default() }),
          ..Default::default()
        }
      }),
      codegen_options: CodegenOptions {
        minify: do_minify,
        comments: !do_minify,
        ..Default::default()
      },
    },
  );
  Result {
    output: result.codegen_return.code,
    diagnostics: result.diagnostics.into_iter().collect(),
  }
}
