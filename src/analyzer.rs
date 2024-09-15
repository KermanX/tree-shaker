use crate::{
  ast::{AstType2, DeclarationKind},
  builtins::Builtins,
  data::{get_node_ptr, ExtraData, ReferredNodes, StatementVecData},
  entity::{
    dep::EntityDep, entity::Entity, forwarded::ForwardedEntity, label::LabelEntity,
    operations::EntityOpHost, union::UnionEntity, unknown::UnknownEntity,
  },
  scope::{variable_scope::VariableScopes, ScopeContext},
  TreeShakeConfig,
};
use oxc::{
  allocator::Allocator,
  ast::ast::Program,
  semantic::{Semantic, SymbolId},
  span::GetSpan,
};
use rustc_hash::FxHashMap;
use std::mem;

pub struct Analyzer<'a> {
  pub config: TreeShakeConfig,
  pub allocator: &'a Allocator,
  pub sematic: Semantic<'a>,
  pub data: ExtraData<'a>,
  pub referred_nodes: ReferredNodes<'a>,
  pub named_exports: Vec<SymbolId>,
  pub default_export: Option<Entity<'a>>,
  pub symbol_decls: FxHashMap<SymbolId, (DeclarationKind, VariableScopes<'a>, EntityDep)>,
  pub scope_context: ScopeContext<'a>,
  pub pending_labels: Vec<LabelEntity<'a>>,
  pub builtins: Builtins<'a>,
  pub entity_op: EntityOpHost<'a>,
}

impl<'a> Analyzer<'a> {
  pub fn new(config: TreeShakeConfig, allocator: &'a Allocator, sematic: Semantic<'a>) -> Self {
    Analyzer {
      config,
      allocator,
      sematic,
      data: Default::default(),
      referred_nodes: Default::default(),
      named_exports: Vec::new(),
      default_export: None,
      symbol_decls: Default::default(),
      scope_context: ScopeContext::new(),
      pending_labels: Vec::new(),
      builtins: Builtins::new(),
      entity_op: EntityOpHost::new(allocator),
    }
  }

  pub fn exec_program(&mut self, node: &'a Program<'a>) {
    let data = self.load_data::<StatementVecData>(AstType2::Program, node);
    self.exec_statement_vec(data, &node.body);

    debug_assert_eq!(self.scope_context.call_scopes.len(), 1);
    debug_assert_eq!(self.scope_context.variable_scopes.len(), 1);
    debug_assert_eq!(self.scope_context.cf_scopes.len(), 1);

    // Consume exports
    self.default_export.take().map(|entity| entity.consume_as_unknown(self));
    for symbol in self.named_exports.clone() {
      let entity = self.read_symbol(&symbol).clone();
      entity.consume_as_unknown(self);
    }
  }
}

impl<'a> Analyzer<'a> {
  pub fn set_data<T>(&mut self, ast_type: AstType2, node: &'a T, data: impl Default + 'a) {
    let key = (ast_type, get_node_ptr(node));
    self.data.insert(key, unsafe { mem::transmute(Box::new(data)) });
  }

  pub fn load_data<D: Default + 'a>(
    &mut self,
    ast_type: AstType2,
    node: &'a impl GetSpan,
  ) -> &'a mut D {
    let key = (ast_type, get_node_ptr(node));
    let boxed =
      self.data.entry(key).or_insert_with(|| unsafe { mem::transmute(Box::new(D::default())) });
    unsafe { mem::transmute(boxed.as_mut()) }
  }
}

impl<'a> Analyzer<'a> {
  pub fn declare_symbol(
    &mut self,
    symbol: SymbolId,
    decl_dep: impl Into<EntityDep>,
    exporting: bool,
    kind: DeclarationKind,
    value: Option<Entity<'a>>,
  ) {
    if exporting {
      self.named_exports.push(symbol);
    }
    let variable_scopes = if kind.is_var() {
      let index = self.call_scope().variable_scope_index;
      self.scope_context.variable_scopes[..index + 1].to_vec()
    } else {
      self.scope_context.variable_scopes.clone()
    };
    variable_scopes.last().unwrap().borrow_mut().declare(kind, symbol, value);
    self.symbol_decls.insert(symbol, (kind, variable_scopes, decl_dep.into()));
  }

  pub fn init_symbol(&mut self, symbol: SymbolId, value: Entity<'a>) {
    let variable_scopes = &self.symbol_decls.get(&symbol).unwrap().1;
    variable_scopes.last().unwrap().borrow_mut().init(symbol, value);
  }

  pub fn read_symbol(&mut self, symbol: &SymbolId) -> Entity<'a> {
    let (_, variable_scopes, _) = self.symbol_decls.get(symbol).unwrap().clone();
    let variable_scope = variable_scopes.last().unwrap().borrow();
    let target_cf_scope = self.find_first_different_cf_scope(&variable_scope.cf_scopes);
    let val = variable_scope.read(self, symbol).1;
    self.mark_exhaustive_read(&val, *symbol, target_cf_scope);
    val
  }

  pub fn write_symbol(&mut self, symbol: &SymbolId, new_val: Entity<'a>) {
    let (kind, variable_scopes, decl_dep) = self.symbol_decls.get(symbol).unwrap();
    if kind.is_const() {
      // TODO: throw warning
    }
    let decl_variable_scope = variable_scopes.last().unwrap().clone();
    let variable_scope_ref = decl_variable_scope.borrow();
    let variable_scope_cf_scopes = &variable_scope_ref.cf_scopes;
    let target_cf_scope = self.find_first_different_cf_scope(variable_scope_cf_scopes);
    let target_variable_scope = self.find_first_different_variable_scope(variable_scopes);
    let mut deps = self.scope_context.variable_scopes[target_variable_scope..]
      .iter()
      .filter_map(|scope| scope.borrow().dep.clone())
      .collect::<Vec<_>>();
    deps.push(decl_dep.clone());
    let dep = EntityDep::from(deps);
    let (is_consumed_exhaustively, old_val) = variable_scope_ref.read(self, symbol);
    if is_consumed_exhaustively {
      drop(variable_scope_ref);
      new_val.consume_as_unknown(self);
    } else {
      let entity_to_set = if self.mark_exhaustive_write(&old_val, symbol.clone(), target_cf_scope) {
        drop(variable_scope_ref);
        self.refer_dep(dep);
        old_val.consume_as_unknown(self);
        new_val.consume_as_unknown(self);
        (true, UnknownEntity::new_unknown())
      } else {
        let indeterminate =
          self.is_relatively_indeterminate(target_cf_scope, variable_scope_cf_scopes);
        drop(variable_scope_ref);
        (
          false,
          ForwardedEntity::new(
            if indeterminate { UnionEntity::new(vec![old_val.clone(), new_val]) } else { new_val },
            dep,
          ),
        )
      };
      decl_variable_scope.borrow_mut().write(*symbol, entity_to_set);
    }
  }

  pub fn refer_global(&mut self) {
    if self.config.unknown_global_side_effects {
      self.may_throw();
      let deps = self
        .scope_context
        .variable_scopes
        .iter()
        .filter_map(|scope| scope.borrow().dep.clone())
        .collect::<Vec<_>>();
      self.refer_dep(deps);
    }
  }
}
