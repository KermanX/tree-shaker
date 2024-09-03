use insta::{assert_snapshot, glob};
use oxc::{allocator::Allocator, minifier::MinifierOptions, span::SourceType};
use std::fs;

use crate::TreeShakeOptions;

fn tree_shake(input: String) -> String {
  let do_minify = input.contains("@minify");
  let result = crate::tree_shake(TreeShakeOptions {
    allocator: &Allocator::default(),
    source_type: SourceType::default().with_module(true).with_always_strict(true),
    source_text: input,
    minify: do_minify.then(|| MinifierOptions::default()),
    eval_mode: false,
  });
  result.codegen_return.source_text
}

#[test]
fn test() {
  glob!("fixtures/**/*.js", |path| {
    let input = fs::read_to_string(path).unwrap();
    assert_snapshot!(tree_shake(input));
  });
}
