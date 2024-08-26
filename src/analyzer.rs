use crate::{
  ast::AstType2,
  data::{ExtraData, ReferredNodes},
  entity::{
    dep::{EntityDep, EntityDepNode},
    entity::Entity,
  },
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
  pub(crate) exports: Vec<SymbolId>,
  pub(crate) scope_context: ScopeContext<'a>,
  pub(crate) current_label: Option<&'a str>,
}

impl<'a> Analyzer<'a> {
  pub(crate) fn new(allocator: &'a Allocator, sematic: Semantic<'a>) -> Self {
    Analyzer {
      allocator,
      sematic,
      data: Default::default(),
      referred_nodes: Default::default(),
      exports: Vec::new(),
      scope_context: ScopeContext::new(),
      current_label: None,
    }
  }

  pub(crate) fn exec_program(&mut self, ast: &'a Program<'a>) {
    for statement in &ast.body {
      self.exec_statement(statement);
    }

    debug_assert_eq!(self.scope_context.function_scopes.len(), 1);
    debug_assert_eq!(self.scope_context.variable_scopes.len(), 1);
    debug_assert_eq!(self.scope_context.cf_scopes.len(), 1);

    for symbol in self.exports.clone() {
      let entity = self.get_symbol(&symbol).clone();
      entity.consume_as_unknown(self);
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
}

impl<'a> Analyzer<'a> {
  pub fn declare_symbol(&mut self, symbol: SymbolId, entity: Entity<'a>, exporting: bool) {
    if exporting {
      self.exports.push(symbol);
    }
    self.variable_scope_mut().declare(symbol, entity)
  }

  pub(crate) fn new_entity_dep(&self, node: EntityDepNode<'a>) -> EntityDep<'a> {
    EntityDep {
      node,
      scope_path: self.scope_context.variable_scopes.iter().map(|x| x.id).collect(),
    }
  }

  pub(crate) fn get_symbol(&self, symbol: &SymbolId) -> &Entity<'a> {
    for scope in self.scope_context.variable_scopes.iter().rev() {
      if let Some(entity) = scope.get(symbol) {
        return entity;
      }
    }
    panic!("Unexpected undeclared Symbol {:?}", self.sematic.symbols().get_name(*symbol));
  }

  pub(crate) fn refer_dep(&mut self, dep: &EntityDep<'a>) {
    self.referred_nodes.insert(dep.node);

    let mut diff = false;
    for (i, scope) in self.scope_context.variable_scopes.iter_mut().enumerate() {
      if diff {
        scope.has_effect = true;
      } else if dep.scope_path.get(i).is_some_and(|id| *id != scope.id) {
        diff = true;
        scope.has_effect = false;
      }
    }
  }

  pub(crate) fn refer_global_dep(&mut self) {
    for scope in self.scope_context.variable_scopes.iter_mut() {
      scope.has_effect = true;
    }
  }
}
