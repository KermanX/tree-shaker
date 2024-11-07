use insta::{assert_snapshot, glob};
use oxc::{codegen::CodegenOptions, minifier::MinifierOptions};
use std::fs;
use crate::{TreeShakeConfig, TreeShakeOptions};

fn tree_shake(input: String) -> String {
  let do_minify = input.contains("@minify");
  let result = crate::tree_shake(
    input,
    TreeShakeOptions {
      config: TreeShakeConfig::default(),
      minify_options: do_minify.then(|| MinifierOptions::default()),
      codegen_options: CodegenOptions::default(),
      logging: false,
    },
  );
  result.codegen_return.code
}

#[test]
fn test() {
  glob!("fixtures/**/*.js", |path| {
    let input = fs::read_to_string(path).unwrap();
    let mut settings = insta::Settings::clone_current();
    settings.set_prepend_module_to_snapshot(false);
    settings.bind(|| {
      assert_snapshot!(tree_shake(input));
    })
  });
}
