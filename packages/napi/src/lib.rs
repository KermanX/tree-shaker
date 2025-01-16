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
  let result = tree_shaker::tree_shake(
    source_text,
    tree_shaker::TreeShakeOptions {
      config: match preset.as_str() {
        "safest" => tree_shaker::TreeShakeConfig::safest(),
        "recommended" => tree_shaker::TreeShakeConfig::recommended(),
        "smallest" => tree_shaker::TreeShakeConfig::smallest(),
        "disabled" => tree_shaker::TreeShakeConfig::disabled(),
        _ => panic!("Invalid tree shake option {}", preset),
      },
      minify_options: minify.then(|| MinifierOptions { mangle: None, ..Default::default() }),
      codegen_options: CodegenOptions { minify, ..Default::default() },
    },
  );
  TreeShakeResultBinding {
    output: result.codegen_return.code,
    diagnostics: result.diagnostics.into_iter().collect(),
  }
}