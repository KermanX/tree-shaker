use crate::{
  ast::AstType2,
  builtins::Builtins,
  data::{ExtraData, ReferredNodes},
  entity::{
    dep::{EntityDep, EntityDepNode},
    entity::Entity,
    forwarded::ForwardedEntity,
    label::LabelEntity,
    operations::EntityOpHost,
    union::UnionEntity,
  },
  scope::ScopeContext,
};
use oxc::{
  allocator::Allocator,
  ast::ast::Program,
  semantic::{ScopeId, Semantic, SymbolId},
  span::{GetSpan, Span},
};
use rustc_hash::FxHashMap;
use std::mem;

pub(crate) struct Analyzer<'a> {
  pub allocator: &'a Allocator,
  pub sematic: Semantic<'a>,
  pub data: ExtraData<'a>,
  pub referred_nodes: ReferredNodes<'a>,
  pub exports: Vec<SymbolId>,
  pub symbol_decls: FxHashMap<SymbolId, (ScopeId, EntityDep<'a>)>,
  pub scope_context: ScopeContext<'a>,
  pub pending_labels: Vec<LabelEntity<'a>>,
  pub builtins: Builtins<'a>,
  pub entity_op: EntityOpHost<'a>,
}

impl<'a> Analyzer<'a> {
  pub(crate) fn new(allocator: &'a Allocator, sematic: Semantic<'a>) -> Self {
    Analyzer {
      allocator,
      sematic,
      data: Default::default(),
      referred_nodes: Default::default(),
      exports: Vec::new(),
      symbol_decls: Default::default(),
      scope_context: ScopeContext::new(),
      pending_labels: Vec::new(),
      builtins: Builtins::new(),
      entity_op: EntityOpHost::new(allocator),
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
  pub fn declare_symbol(
    &mut self,
    symbol: SymbolId,
    dep: EntityDep<'a>,
    entity: Entity<'a>,
    exporting: bool,
  ) {
    if exporting {
      self.exports.push(symbol);
    }
    let variable_scope = self.variable_scope_mut();
    variable_scope.declare(symbol, entity);
    let scope_id = variable_scope.id;
    self.symbol_decls.insert(symbol, (scope_id, dep));
  }

  pub(crate) fn new_entity_dep(&self, node: EntityDepNode<'a>) -> EntityDep<'a> {
    EntityDep { node, scope_path: self.variable_scope_path() }
  }

  pub(crate) fn get_symbol(&self, symbol: &SymbolId) -> &Entity<'a> {
    for scope in self.scope_context.variable_scopes.iter().rev() {
      if let Some(entity) = scope.get(symbol) {
        return entity;
      }
    }
    panic!("Unexpected undeclared Symbol {:?}", self.sematic.symbols().get_name(*symbol));
  }

  pub(crate) fn set_symbol(&mut self, symbol: &SymbolId, new_val: Entity<'a>) {
    let (scope_id, dep) = self.symbol_decls.get(symbol).unwrap();
    let variable_scope = self.get_variable_scope_by_id(*scope_id);
    let indeterminate = self.is_relative_indeterminate(variable_scope.cf_scope_id);
    let old_val = variable_scope.get(symbol).unwrap();
    let entity = ForwardedEntity::new(
      if indeterminate { UnionEntity::new(vec![old_val.clone(), new_val]) } else { new_val },
      dep.clone(),
    );
    self.get_variable_scope_by_id_mut(*scope_id).set(*symbol, entity).unwrap();
  }

  pub(crate) fn refer_dep(&mut self, dep: &EntityDep<'a>) {
    self.referred_nodes.insert(dep.node);

    let mut diff = false;
    for (i, scope) in self.scope_context.variable_scopes.iter_mut().enumerate() {
      if diff || dep.scope_path.get(i) != Some(&scope.id) {
        diff = true;
        scope.has_effect = true;
      }
    }
  }

  pub(crate) fn refer_global_dep(&mut self) {
    for scope in self.scope_context.variable_scopes.iter_mut() {
      scope.has_effect = true;
    }
  }
}
