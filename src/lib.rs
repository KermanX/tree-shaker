mod analyzer;
mod ast;
mod builtins;
mod config;
mod data;
mod effect_builder;
mod entity;
mod logger;
mod nodes;
mod scope;
mod transformer;
mod utils;

#[cfg(test)]
mod tests;

use analyzer::Analyzer;
pub use config::TreeShakeConfig;
use data::Diagnostics;
use logger::Logger;
use oxc::{
  allocator::Allocator,
  ast::AstBuilder,
  codegen::{CodeGenerator, CodegenOptions, CodegenReturn},
  minifier::{Minifier, MinifierOptions, MinifierReturn},
  parser::Parser,
  semantic::SemanticBuilder,
  span::SourceType,
};
use transformer::Transformer;
use utils::{transform_eval_mode_decode, transform_eval_mode_encode};

pub struct TreeShakeOptions<'a> {
  pub config: TreeShakeConfig,
  pub allocator: &'a Allocator,
  pub source_type: SourceType,
  pub source_text: String,
  pub tree_shake: bool,
  pub minify: Option<MinifierOptions>,
  pub code_gen: CodegenOptions,
  pub eval_mode: bool,
  pub logging: bool,
}

pub struct TreeShakeReturn {
  pub minifier_return: Option<MinifierReturn>,
  pub codegen_return: CodegenReturn,
  pub diagnostics: Diagnostics,
  pub logs: Vec<String>,
}

pub fn tree_shake<'a>(options: TreeShakeOptions<'a>) -> TreeShakeReturn {
  let TreeShakeOptions {
    config,
    allocator,
    source_type,
    source_text,
    tree_shake,
    minify,
    code_gen,
    eval_mode,
    logging,
  } = options;

  let ast_builder = AstBuilder::new(allocator);
  let logger = logging.then(|| &*allocator.alloc(Logger::new()));

  let parser = Parser::new(&allocator, source_text.as_str(), source_type);
  let mut ast = allocator.alloc(parser.parse().program);

  if eval_mode {
    transform_eval_mode_encode(&ast_builder, ast);
  }

  let semantic_builder = SemanticBuilder::new(source_text.as_str());
  let semantic = semantic_builder.build(ast).semantic;
  let mut diagnostics = Diagnostics::default();

  if tree_shake {
    // Step 1: Analyze the program
    let mut analyzer = Analyzer::new(config, allocator, semantic, &mut diagnostics, logger);
    analyzer.exec_program(ast);

    // Step 2: Remove dead code (transform)
    let transformer = Transformer::new(analyzer);
    ast = allocator.alloc(transformer.transform_program(ast));
  }

  // Step 3: Minify
  let minifier_return = minify.map(|options| {
    let minifier = Minifier::new(options);
    minifier.build(&allocator, ast)
  });

  if eval_mode {
    transform_eval_mode_decode(&ast_builder, ast);
  }

  // Step 4: Generate output
  let codegen = CodeGenerator::new().with_options(code_gen);
  let codegen_return = codegen.build(ast);

  TreeShakeReturn {
    minifier_return,
    codegen_return,
    diagnostics,
    logs: logger.map(|l| l.serialize()).unwrap_or_default(),
  }
}
