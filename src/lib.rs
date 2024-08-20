mod builtins;
mod context;
mod effect_builder;
mod entity;
mod nodes;
mod symbol;
mod utils;

use context::Context;
use entity::Entity;
use oxc::{
  allocator::Allocator,
  ast::{
    ast::{Expression, Function, NumberBase, Program, Statement},
    AstBuilder,
  },
  codegen::{CodeGenerator, CodegenReturn},
  minifier::{Minifier, MinifierOptions, MinifierReturn},
  parser::Parser,
  semantic::{Semantic, SemanticBuilder, SymbolId},
  span::{GetSpan, SourceType, Span, SPAN},
};
use rustc_hash::FxHashMap;
use std::{any::Any, mem};
use symbol::SymbolSource;

pub(crate) struct TreeShaker<'a> {
  pub sematic: Semantic<'a>,
  pub ast_builder: AstBuilder<'a>,
  pub functions: FxHashMap<Span, &'a Function<'a>>,
  symbol_source: FxHashMap<SymbolId, SymbolSource<'a>>,
  pub data: FxHashMap<Span, Box<dyn Any>>,
  pub context: Context,
}

impl<'a> TreeShaker<'a> {
  pub fn new(allocator: &'a Allocator, sematic: Semantic<'a>) -> Self {
    TreeShaker {
      sematic,
      ast_builder: AstBuilder::new(allocator),
      functions: FxHashMap::default(),
      symbol_source: FxHashMap::default(),
      data: FxHashMap::default(),
      context: Context::new(),
    }
  }

  pub fn execute_program(&mut self, ast: &'a Program<'a>) {
    for statement in &ast.body {
      self.exec_statement(statement);
    }
  }

  pub fn transform_program(&mut self, ast: &'a mut Program<'a>) -> Program<'a> {
    let Program { span, source_type, hashbang, directives, body: old_statements, .. } =
      mem::replace(
        ast,
        self.ast_builder.program(
          SPAN,
          SourceType::default(),
          None,
          self.ast_builder.vec(),
          self.ast_builder.vec(),
        ),
      );
    let mut new_statements = self.ast_builder.vec::<Statement>();
    for statement in old_statements {
      let new_statement = self.transform_statement(statement);
      if let Some(new_statement) = new_statement {
        new_statements.push(new_statement);
      }
    }
    self.ast_builder.program(span, source_type, hashbang, directives, new_statements)
  }
}

impl<'a> TreeShaker<'a> {
  pub(crate) fn load_data_from_span<D: Default + 'static>(&mut self, span: Span) -> &'a mut D {
    if !self.data.contains_key(&span) {
      self.data.insert(span, Box::new(D::default()));
    }
    let x = self.data.get_mut(&span).unwrap();
    let t = x.downcast_mut::<D>().unwrap();
    unsafe { mem::transmute(t) }
  }

  pub(crate) fn load_data<D: Default + 'static>(&mut self, node: &dyn GetSpan) -> &'a mut D {
    self.load_data_from_span(node.span())
  }

  pub(crate) fn entity_to_expression(&self, span: Span, value: &Entity) -> Option<Expression<'a>> {
    match value {
      Entity::StringLiteral(s) => Some(self.ast_builder.expression_string_literal(span, s.clone())),
      Entity::NumberLiteral(n) => Some(self.ast_builder.expression_numeric_literal(
        span,
        *n,
        n.to_string(),
        NumberBase::Decimal,
      )),
      Entity::BooleanLiteral(b) => Some(self.ast_builder.expression_boolean_literal(span, *b)),
      Entity::Null => Some(self.ast_builder.expression_null_literal(span)),
      Entity::Undefined => {
        Some(self.ast_builder.expression_identifier_reference(span, "undefined"))
      }
      _ => None,
    }
  }
}

pub struct TreeShakeReturn {
  pub minifier_return: MinifierReturn,
  pub codegen_return: CodegenReturn,
}

pub fn tree_shake(source_text: &str) -> TreeShakeReturn {
  let allocator = Allocator::default();
  let source_type = SourceType::default();
  let parser = Parser::new(&allocator, source_text, source_type);
  let ast1 = allocator.alloc(parser.parse().program);
  // TODO: Reuse the AST
  let parser = Parser::new(&allocator, source_text, source_type);
  let ast2 = allocator.alloc(parser.parse().program);
  let sematic_builder = SemanticBuilder::new(source_text, source_type);
  let sematic = sematic_builder.build(&ast1).semantic;
  let mut tree_shaker = TreeShaker::new(&allocator, sematic);

  // Step 1: Execute the program
  tree_shaker.execute_program(ast1);

  // Step 2: Execute exports
  // TODO:

  // Step 3: Remove dead code (transform)
  let mut program = tree_shaker.transform_program(ast2);

  // Step 4: Minify
  let minifier = Minifier::new(MinifierOptions::default());
  let minifier_return = minifier.build(&allocator, &mut program);

  // Step 5: Generate output
  let codegen = CodeGenerator::new();
  let codegen_return = codegen.build(&program);

  TreeShakeReturn { minifier_return, codegen_return }
}
