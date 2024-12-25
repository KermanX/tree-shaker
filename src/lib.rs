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
pub use tree_shaker::TreeShakeOptions;
use tree_shaker::{TreeShakeReturn, TreeShaker};
pub use utils::ast;
pub use utils::dep_id::{self as dep};

pub fn tree_shake(source_text: String, options: TreeShakeOptions) -> TreeShakeReturn {
  let allocator = Allocator::default();
  let tree_shaker = TreeShaker::new(&allocator, options);
  tree_shaker.tree_shake(source_text)
}
