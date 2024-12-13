#![deny(clippy::all)]

use oxc::{codegen::CodegenOptions, minifier::MinifierOptions};

#[macro_use]
extern crate napi_derive;

#[napi]
pub struct TreeShakeResultBinding {
  pub output: String,
  pub diagnostics: Vec<String>,
}

#[napi(
  ts_args_type = "input: string, preset: 'safest' | 'recommended' | 'smallest' | 'disabled', minify: boolean"
)]
pub fn tree_shake(source_text: String, preset: String, minify: bool) -> TreeShakeResultBinding {
  let result = tree_shake::tree_shake(
    source_text,
    tree_shake::TreeShakeOptions {
      config: match preset.as_str() {
        "safest" => tree_shake::TreeShakeConfig::safest(),
        "recommended" => tree_shake::TreeShakeConfig::recommended(),
        "smallest" => tree_shake::TreeShakeConfig::smallest(),
        "disabled" => tree_shake::TreeShakeConfig::disabled(),
        _ => panic!("Invalid tree shake option {}", preset),
      },
      minify_options: minify.then(MinifierOptions::default),
      codegen_options: CodegenOptions { minify, ..Default::default() },
    },
  );
  TreeShakeResultBinding {
    output: result.codegen_return.code,
    diagnostics: result.diagnostics.into_iter().collect(),
  }
}
