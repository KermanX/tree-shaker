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
  semantic::{Semantic, SymbolId},
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
  pub decls_deps: FxHashMap<SymbolId, EntityDep<'a>>,
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
      decls_deps: Default::default(),
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
    self.decls_deps.insert(symbol, dep);
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

  pub(crate) fn set_symbol(&mut self, symbol: &SymbolId, new_val: Entity<'a>) {
    let indeterminate = self.cf_scope().is_indeterminate();
    let decl_dep = self.decls_deps.get(symbol).unwrap();
    for scope in self.scope_context.variable_scopes.iter_mut().rev() {
      if let Some(old_val) = scope.get(symbol) {
        let entity = ForwardedEntity::new(
          if indeterminate { UnionEntity::new(vec![old_val.clone(), new_val]) } else { new_val },
          decl_dep.clone(),
        );
        scope.set(*symbol, entity).unwrap();
        return;
      }
    }
    panic!("Unexpected undeclared Symbol {:?}", self.sematic.symbols().get_name(*symbol));
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
