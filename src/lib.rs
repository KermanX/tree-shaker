mod analyzer;
mod builtins;
mod config;
mod consumable;
mod entity;
mod mangling;
mod nodes;
mod scope;
mod transformer;
mod tree_shaker;
mod utils;

#[cfg(test)]
mod tests;

use analyzer::Analyzer;
pub use config::TreeShakeConfig;
use oxc::allocator::Allocator;
use oxc::parser::Parser;
use oxc::span::SourceType;
pub use tree_shaker::TreeShakeOptions;
use tree_shaker::{TreeShakeReturn, TreeShaker};
pub use utils::ast;
pub use utils::dep_id::{self as dep};

pub fn tree_shake(source_text: String, options: TreeShakeOptions) -> TreeShakeReturn {
  let source_type = SourceType::mjs().with_jsx(options.config.jsx.is_enabled());
  let allocator = Allocator::default();
  let tree_shaker = TreeShaker::new(&allocator, options);

  let parser = Parser::new(&allocator, allocator.alloc(source_text), source_type);
  let parsed = allocator.alloc(parser.parse());
  let errors = parsed.errors.iter().map(|e| format!("{}", e)).collect::<Vec<_>>();

  let mut result = tree_shaker.tree_shake(&mut parsed.program);
  result.diagnostics.extend(errors);
  result
}
