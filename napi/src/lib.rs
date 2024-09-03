#![deny(clippy::all)]

use oxc::{allocator::Allocator, minifier::MinifierOptions, span::SourceType};

#[macro_use]
extern crate napi_derive;

#[napi]
pub fn tree_shake(input: String, do_minify: bool, eval_mode: bool) -> String {
  let result = tree_shake::tree_shake(tree_shake::TreeShakeOptions {
    allocator: &Allocator::default(),
    source_type: SourceType::default().with_module(true).with_always_strict(true),
    source_text: input,
    minify: do_minify.then(|| MinifierOptions::default()),
    eval_mode,
  });
  result.codegen_return.source_text
}
