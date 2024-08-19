mod entity;
mod nodes;

use entity::Entity;
use oxc::{
  allocator::Allocator,
  ast::ast::{Declaration, Program},
  parser::Parser,
  semantic::{Semantic, SemanticBuilder, SymbolId},
  span::{GetSpan, SourceType, Span},
};
use rustc_hash::FxHashMap;
use std::{any::Any, mem};

pub struct TreeShaker<'a> {
  allocator: &'a Allocator,
  ast: Program<'a>,
  implementation: TreeShakerImpl<'a>,
}

pub(crate) struct TreeShakerImpl<'a> {
  pub sematic: Semantic<'a>,
  pub declaration: FxHashMap<SymbolId, &'a Declaration<'a>>,
  pub current_declaration: Option<&'a Declaration<'a>>,
  pub data: FxHashMap<Span, Box<dyn Any>>,
}

impl<'a> TreeShaker<'a> {
  pub fn new(allocator: &'a Allocator, source_text: &'a str) -> Self {
    let source_type = SourceType::default();
    let parser = Parser::new(&allocator, source_text, source_type);
    let ast = parser.parse().program;
    let sematic_builder = SemanticBuilder::new(source_text, source_type);
    let sematic = sematic_builder.build(&ast).semantic;
    TreeShaker {
      allocator,
      ast,
      implementation: TreeShakerImpl {
        sematic,
        declaration: FxHashMap::default(),
        current_declaration: None,
        data: FxHashMap::default(),
      },
    }
  }

  pub fn tree_shake(&'a mut self) {
    // Step 1: Execute the program
    for statement in &self.ast.body {
      self.implementation.exec_statement(statement);
    }

    // Step 2: Execute exports
    // TODO:

    // Step 3: Remove dead code
    // TODO:

    // Step 4: Minify
    // TODO:
  }
}

impl<'a> TreeShakerImpl<'a> {
  pub(crate) fn load_data<D: Default + 'static>(&mut self, node: &dyn GetSpan) -> &'a mut D {
    if !self.data.contains_key(&node.span()) {
      self.data.insert(node.span(), Box::new(D::default()));
    }
    let x = self.data.get_mut(&node.span()).unwrap();
    let t = x.downcast_mut::<D>().unwrap();
    unsafe { mem::transmute(t) }
  }

  pub(crate) fn declare_symbol(&mut self, symbol_id: SymbolId) {
    self.current_declaration.map(|declaration| {
      self.declaration.insert(symbol_id, declaration);
    });
  }

  pub(crate) fn read_symbol(&mut self, symbol_id: SymbolId) -> Entity {
    let declaration = self.declaration.get(&symbol_id).expect("Missing declaration");
    self.exec_declaration(declaration, Some(symbol_id)).unwrap()
  }

  pub(crate) fn write_symbol(&mut self, symbol_id: SymbolId, entity: Entity) {
    todo!()
  }
}
