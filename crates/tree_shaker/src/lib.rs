mod analyzer;
mod builtins;
mod config;
mod consumable;
mod entity;
mod mangling;
mod nodes;
mod transformer;
mod tree_shaker;
mod utils;

use analyzer::Analyzer;
use oxc::{allocator::Allocator, parser::Parser, span::SourceType};
use utils::{
  ast,
  dep_id::{self as dep},
};

pub use config::{TreeShakeConfig, TreeShakeJsxPreset};
pub use tree_shaker::{TreeShakeOptions, TreeShakeReturn, TreeShaker};

pub fn tree_shake(source_text: String, options: TreeShakeOptions) -> TreeShakeReturn {
  let source_type = SourceType::mjs().with_jsx(options.config.jsx.is_enabled());
  let allocator = Allocator::default();
  let tree_shaker = TreeShaker::new(&allocator, options);

  let parser = Parser::new(&allocator, &source_text, source_type);
  let mut parsed = parser.parse();
  let errors = parsed.errors.iter().map(|e| format!("{}", e)).collect::<Vec<_>>();

  let mut result = tree_shaker.tree_shake(&mut parsed.program);
  result.diagnostics.extend(errors);
  result
}
