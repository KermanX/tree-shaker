mod analyzer;
mod ast;
mod builtins;
mod data;
mod effect_builder;
mod entity;
mod nodes;
mod scope;
#[cfg(test)]
mod tests;
mod transformer;
mod utils;

use analyzer::Analyzer;
use oxc::{
  allocator::Allocator,
  codegen::{CodeGenerator, CodegenReturn},
  minifier::{Minifier, MinifierOptions, MinifierReturn},
  parser::Parser,
  semantic::SemanticBuilder,
  span::SourceType,
};
use transformer::Transformer;

pub struct TreeShakeReturn {
  pub minifier_return: Option<MinifierReturn>,
  pub codegen_return: CodegenReturn,
}

pub fn tree_shake(source_text: &str, do_minify: bool) -> TreeShakeReturn {
  let allocator = Allocator::default();
  let source_type = SourceType::default();
  let parser = Parser::new(&allocator, source_text, source_type);
  let ast1 = allocator.alloc(parser.parse().program);
  let sematic_builder = SemanticBuilder::new(source_text, source_type);
  let sematic = sematic_builder.build(ast1).semantic;

  // Step 1: Analyze the program
  let mut analyzer = Analyzer::new(&allocator, sematic);
  analyzer.exec_program(ast1);

  // Step 3: Remove dead code (transform)
  let mut transformer = Transformer::new(analyzer);
  // TODO: Reuse the AST
  let parser2 = Parser::new(&allocator, source_text, source_type);
  let ast2 = parser2.parse().program;
  let mut program = transformer.transform_program(ast2);

  // Step 4: Minify
  let minifier_return = do_minify.then(|| {
    let minifier = Minifier::new(MinifierOptions::default());
    minifier.build(&allocator, &mut program)
  });

  // Step 5: Generate output
  let codegen = CodeGenerator::new();
  let codegen_return = codegen.build(&program);

  TreeShakeReturn { minifier_return, codegen_return }
}
