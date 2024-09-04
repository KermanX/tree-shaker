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
  ast::AstBuilder,
  codegen::{CodeGenerator, CodegenReturn},
  minifier::{Minifier, MinifierOptions, MinifierReturn},
  parser::Parser,
  semantic::SemanticBuilder,
  span::SourceType,
};
use transformer::Transformer;
use utils::{transform_eval_mode_decode, transform_eval_mode_encode};

pub struct TreeShakeOptions<'a> {
  pub allocator: &'a Allocator,
  pub source_type: SourceType,
  pub source_text: String,
  pub minify: Option<MinifierOptions>,
  pub eval_mode: bool,
}

pub struct TreeShakeReturn {
  pub minifier_return: Option<MinifierReturn>,
  pub codegen_return: CodegenReturn,
}

pub fn tree_shake<'a>(options: TreeShakeOptions<'a>) -> TreeShakeReturn {
  let TreeShakeOptions { allocator, source_type, source_text, minify, eval_mode } = options;

  let ast_builder = AstBuilder::new(allocator);

  let parser = Parser::new(&allocator, source_text.as_str(), source_type);
  let ast = allocator.alloc(parser.parse().program);

  if eval_mode {
    transform_eval_mode_encode(&ast_builder, ast);
  }

  let sematic_builder = SemanticBuilder::new(source_text.as_str(), source_type);
  let sematic = sematic_builder.build(ast).semantic;

  // Step 1: Analyze the program
  let mut analyzer = Analyzer::new(&allocator, sematic);
  analyzer.exec_program(ast);

  // Step 3: Remove dead code (transform)
  let transformer = Transformer::new(analyzer);
  let mut program = transformer.transform_program(ast);

  // Step 4: Minify
  let minifier_return = minify.map(|options| {
    let minifier = Minifier::new(options);
    minifier.build(&allocator, &mut program)
  });

  if eval_mode {
    transform_eval_mode_decode(&ast_builder, &mut program);
  }

  // Step 5: Generate output
  let codegen = CodeGenerator::new();
  let codegen_return = codegen.build(&program);

  TreeShakeReturn { minifier_return, codegen_return }
}
