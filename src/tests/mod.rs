use insta::{assert_snapshot, glob};
use std::fs;

fn tree_shake(input: &str) -> String {
  let do_minify = input.contains("@minify");
  let result = crate::tree_shake(input, do_minify);
  result.codegen_return.source_text
}

#[test]
fn test() {
  glob!("fixtures/**/*.js", |path| {
    let input = fs::read_to_string(path).unwrap();
    assert_snapshot!(tree_shake(&input));
  });
}
