mod analyzer;
mod builtins;
mod config;
mod consumable;
mod entity;
mod nodes;
mod scope;
mod transformer;
mod tree_shaker;
mod utils;

#[cfg(test)]
mod tests;

use analyzer::Analyzer;
pub use config::TreeShakeConfig;
use oxc::{allocator::Allocator, codegen::CodegenReturn, minifier::MinifierReturn};
pub use tree_shaker::TreeShakeOptions;
use tree_shaker::TreeShaker;
pub use utils::ast;
pub use utils::dep_id::{self as dep};
use utils::Diagnostics;

pub struct TreeShakeReturn {
  pub minifier_return: Option<MinifierReturn>,
  pub codegen_return: CodegenReturn,
  pub diagnostics: Diagnostics,
}

pub fn tree_shake<'a>(source_text: String, options: TreeShakeOptions) -> TreeShakeReturn {
  let allocator = Allocator::default();
  let tree_shaker = TreeShaker::new(&allocator, options);
  let (minifier_return, codegen_return) = tree_shaker.tree_shake(source_text);

  TreeShakeReturn { minifier_return, codegen_return, diagnostics: tree_shaker.0.diagnostics.take() }
}
