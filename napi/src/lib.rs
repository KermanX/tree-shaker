#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

#[napi]
pub fn tree_shake(input: String, do_minify: bool) -> String {
  let result = tree_shake::tree_shake(input.as_str(), do_minify);
  result.codegen_return.source_text
}
