use crate::{
  ast::AstType2,
  data::{ExtraData, ReferredNodes},
  entity::{dep::EntityDep, entity::Entity},
  scope::ScopeContext,
};
use oxc::{
  allocator::Allocator,
  ast::ast::Program,
  semantic::{Semantic, SymbolId},
  span::{GetSpan, Span},
};
use rustc_hash::FxHashMap;
use std::mem;

pub(crate) struct Analyzer<'a> {
  pub(crate) allocator: &'a Allocator,
  pub(crate) sematic: Semantic<'a>,
  pub(crate) data: ExtraData<'a>,
  pub(crate) referred_nodes: ReferredNodes<'a>,
  pub(crate) indeterminate: bool,
  pub(crate) exporting: bool,
  pub(crate) exports: Vec<SymbolId>,
  pub(crate) scope_context: ScopeContext<'a>,
}

impl<'a> Analyzer<'a> {
  pub(crate) fn new(allocator: &'a Allocator, sematic: Semantic<'a>) -> Self {
    Analyzer {
      allocator,
      sematic,
      data: Default::default(),
      referred_nodes: Default::default(),
      indeterminate: false,
      exporting: false,
      exports: Vec::new(),
      scope_context: ScopeContext::new(),
    }
  }

  pub(crate) fn exec_program(&mut self, ast: &'a Program<'a>) {
    for statement in &ast.body {
      self.exec_statement(statement);
    }

    for symbol in self.exports.clone() {
      // TODO: Should consume the symbol
      self.consume_entity(&self.variable_scope().get(&symbol));
    }
  }
}

impl<'a> Analyzer<'a> {
  pub(crate) fn set_data_by_span(
    &mut self,
    ast_type: AstType2,
    span: Span,
    data: impl Default + 'a,
  ) {
    let map = self.data.entry(ast_type).or_insert_with(|| FxHashMap::default());
    map.insert(span, unsafe { mem::transmute(Box::new(data)) });
  }

  pub(crate) fn set_data(
    &mut self,
    ast_type: AstType2,
    node: &dyn GetSpan,
    data: impl Default + 'a,
  ) {
    self.set_data_by_span(ast_type, node.span(), data)
  }

  pub(crate) fn load_data_by_span<D: Default + 'a>(
    &mut self,
    ast_type: AstType2,
    span: Span,
  ) -> &'a mut D {
    let map = self.data.entry(ast_type).or_insert_with(|| FxHashMap::default());
    let boxed =
      map.entry(span).or_insert_with(|| unsafe { mem::transmute(Box::new(D::default())) });
    unsafe { mem::transmute(boxed.as_mut()) }
  }

  pub(crate) fn load_data<D: Default + 'a>(
    &mut self,
    ast_type: AstType2,
    node: &dyn GetSpan,
  ) -> &'a mut D {
    self.load_data_by_span(ast_type, node.span())
  }

  pub(crate) fn get_data_by_span<D: Default + 'a>(&self, ast_type: AstType2, span: Span) -> &'a D {
    let existing = self.data.get(&ast_type).and_then(|map| map.get(&span));
    match existing {
      Some(boxed) => unsafe { mem::transmute(boxed.as_ref()) },
      None => self.allocator.alloc(D::default()),
    }
  }

  pub(crate) fn get_data<D: Default + 'a>(&self, ast_type: AstType2, node: &dyn GetSpan) -> &'a D {
    self.get_data_by_span(ast_type, node.span())
  }
}

impl<'a> Analyzer<'a> {
  pub fn declare_symbol(&mut self, symbol: SymbolId, entity: Entity<'a>) {
    if self.exporting {
      self.exports.push(symbol);
    }
    self.variable_scope_mut().declare(symbol, entity)
  }

  pub(crate) fn refer_dep(&mut self, node: EntityDep<'a>) {
    self.referred_nodes.insert(node);
  }

  pub(crate) fn is_referred(&self, node: &EntityDep<'a>) -> bool {
    self.referred_nodes.contains(node)
  }
}
