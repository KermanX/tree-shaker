use crate::{
  analyzer::Analyzer, entity::EntityFactory, logger::Logger, transformer::Transformer,
  TreeShakeConfig,
};
use oxc::{
  allocator::Allocator,
  codegen::{CodeGenerator, CodegenOptions, CodegenReturn},
  minifier::{Minifier, MinifierOptions, MinifierReturn},
  parser::Parser,
  semantic::SemanticBuilder,
  span::SourceType,
};
use std::{cell::RefCell, collections::BTreeSet, rc::Rc};

pub struct TreeShakeOptions {
  pub config: TreeShakeConfig,
  pub minify_options: Option<MinifierOptions>,
  pub codegen_options: CodegenOptions,
  pub logging: bool,
}

pub struct TreeShakerInner<'a> {
  pub allocator: &'a Allocator,
  pub config: &'a TreeShakeConfig,
  pub factory: &'a EntityFactory<'a>,
  pub logger: Option<&'a Logger>,
  pub minify_options: Option<MinifierOptions>,
  pub codegen_options: CodegenOptions,
  pub diagnostics: RefCell<BTreeSet<String>>,
}

#[derive(Clone)]
pub struct TreeShaker<'a>(pub Rc<TreeShakerInner<'a>>);

impl<'a> TreeShaker<'a> {
  pub fn new(allocator: &'a Allocator, options: TreeShakeOptions) -> Self {
    let TreeShakeOptions { config, minify_options, codegen_options, logging, .. } = options;

    Self(Rc::new(TreeShakerInner {
      allocator,
      config: allocator.alloc(config),
      minify_options,
      codegen_options,
      logger: logging.then(|| &*allocator.alloc(Logger::new())),
      factory: allocator.alloc(EntityFactory::new(allocator)),
      diagnostics: Default::default(),
    }))
  }

  pub fn tree_shake(&self, source_text: String) -> (Option<MinifierReturn>, CodegenReturn) {
    let TreeShakerInner { allocator, config, minify_options, codegen_options, logger, .. } =
      self.0.as_ref();

    let parser = Parser::new(allocator, allocator.alloc(source_text), SourceType::mjs());
    let mut ast = allocator.alloc(parser.parse().program);

    if config.enabled {
      let semantic_builder = SemanticBuilder::new();
      let semantic = semantic_builder.build(ast).semantic;

      // Step 1: Analyze the program
      let mut analyzer = Analyzer::new(self.clone(), semantic);
      analyzer.exec_program(ast);

      // Step 2: Remove dead code (transform)
      let transformer = Transformer::new(analyzer);
      ast = allocator.alloc(transformer.transform_program(ast));
    }

    // Step 3: Minify
    let minifier_return = minify_options.map(|options| {
      let minifier = Minifier::new(options);
      minifier.build(&allocator, ast)
    });

    // Step 4: Generate output
    let codegen = CodeGenerator::new().with_options(codegen_options.clone());
    let codegen_return = codegen.build(ast);

    logger.map(|l| l.print_fn_calls());

    (minifier_return, codegen_return)
  }
}
