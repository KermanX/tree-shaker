use crate::{
  ast::{AstType2, DeclarationKind},
  builtins::Builtins,
  data::{get_node_ptr, Diagnostics, ExtraData, ReferredNodes, StatementVecData, VarDeclarations},
  entity::{Consumable, Entity, EntityOpHost, LabelEntity, UnknownEntity},
  scope::{
    exhaustive::TrackerRunner,
    variable_scope::{VariableScope, VariableScopes},
    ScopeContext,
  },
  TreeShakeConfig,
};
use oxc::{
  allocator::Allocator,
  ast::ast::Program,
  semantic::{Semantic, SymbolId},
  span::{GetSpan, Span},
};
use rustc_hash::{FxHashMap, FxHashSet};
use std::{mem, rc::Rc};

pub struct Analyzer<'a> {
  pub config: TreeShakeConfig,
  pub allocator: &'a Allocator,
  pub semantic: Semantic<'a>,
  pub diagnostics: &'a mut Diagnostics,
  pub current_span: Vec<Span>,
  pub data: ExtraData<'a>,
  pub referred_nodes: ReferredNodes<'a>,
  pub var_decls: VarDeclarations<'a>,
  pub named_exports: Vec<SymbolId>,
  pub default_export: Option<Entity<'a>>,
  pub exhaustive_deps: FxHashMap<SymbolId, FxHashSet<TrackerRunner<'a>>>,
  pub scope_context: ScopeContext<'a>,
  pub pending_labels: Vec<LabelEntity<'a>>,
  pub builtins: Builtins<'a>,
  pub entity_op: EntityOpHost<'a>,
}

impl<'a> Analyzer<'a> {
  pub fn new(
    config: TreeShakeConfig,
    allocator: &'a Allocator,
    semantic: Semantic<'a>,
    diagnostics: &'a mut Diagnostics,
  ) -> Self {
    Analyzer {
      config,
      allocator,
      semantic,
      diagnostics,
      current_span: vec![],
      data: Default::default(),
      referred_nodes: Default::default(),
      var_decls: Default::default(),
      named_exports: Vec::new(),
      default_export: None,
      exhaustive_deps: Default::default(),
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
    self.default_export.take().map(|entity| entity.consume(self));
    for symbol in self.named_exports.clone() {
      let entity = self.read_symbol(symbol).unwrap();
      entity.consume(self);
    }
    // Consume uncaught thrown values
    self.call_scope_mut().try_scopes.pop().unwrap().thrown_val().map(|entity| {
      entity.consume(self);
    });
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

  pub fn add_diagnostic(&mut self, message: impl Into<String>) {
    let span = self.current_span.last().unwrap();
    self.diagnostics.insert(message.into() + format!(" at {}-{}", span.start, span.end).as_str());
  }
}

impl<'a> Analyzer<'a> {
  pub fn declare_symbol(
    &mut self,
    symbol: SymbolId,
    decl_dep: impl Into<Consumable<'a>>,
    exporting: bool,
    kind: DeclarationKind,
    fn_value: Option<Entity<'a>>,
  ) {
    if exporting {
      self.named_exports.push(symbol);
    }
    if kind == DeclarationKind::FunctionParameter {
      self.call_scope_mut().args.1.push(symbol);
    }
    if kind == DeclarationKind::Var {
      self.insert_var_decl(symbol);
    }

    let variable_scope = self.get_variable_scope(kind.is_var());

    variable_scope.declare(self, kind, symbol, decl_dep.into(), fn_value);
  }

  pub fn init_symbol(&mut self, symbol: SymbolId, value: Option<Entity<'a>>) {
    let is_function_scope =
      self.semantic.symbols().get_flags(symbol).is_function_scoped_declaration();
    let variable_scope = self.get_variable_scope(is_function_scope);
    variable_scope.init(self, symbol, value);
  }

  fn get_variable_scope(&self, is_function_scope: bool) -> Rc<VariableScope<'a>> {
    if is_function_scope {
      let index = self.call_scope().variable_scope_index;
      self.scope_context.variable_scopes[index].clone()
    } else {
      self.scope_context.variable_scopes.last().unwrap().clone()
    }
  }

  /// `None` for TDZ
  pub fn read_symbol(&mut self, symbol: SymbolId) -> Option<Entity<'a>> {
    for index in (0..self.scope_context.variable_scopes.len()).rev() {
      let scope = self.scope_context.variable_scopes[index].clone();
      if let Some(value) = scope.read(self, symbol) {
        return value;
      }
    }
    self.mark_unresolved_reference(symbol);
    Some(UnknownEntity::new_unknown())
  }

  pub fn write_symbol(&mut self, symbol: SymbolId, new_val: Entity<'a>) {
    for index in (0..self.scope_context.variable_scopes.len()).rev() {
      let scope = self.scope_context.variable_scopes[index].clone();
      if scope.write(self, symbol, &new_val, index) {
        return;
      }
    }
    self.consume(new_val);
    self.mark_unresolved_reference(symbol);
  }

  fn mark_unresolved_reference(&mut self, symbol: SymbolId) {
    if self.semantic.symbols().get_flags(symbol).is_function_scoped_declaration() {
      self.insert_var_decl(symbol);
      self.get_variable_scope(true).mark_untracked(symbol);
    } else {
      self.explicit_throw_unknown("Unresolved identifier reference");
    }
  }

  fn insert_var_decl(&mut self, symbol: SymbolId) {
    let key = self.call_scope().source;
    self.var_decls.entry(key).or_default().insert(symbol);
  }

  pub fn handle_tdz(&mut self, target_cf_scope: usize) {
    if self.has_exhaustive_scope_since(target_cf_scope) {
      self.may_throw();
    } else {
      self.explicit_throw_unknown("Cannot access variable before initialization");
    }
    self.refer_global();
  }

  pub fn refer_global(&mut self) {
    if self.config.unknown_global_side_effects {
      self.may_throw();
      self.refer_to_scope(0);
    }
  }

  pub fn refer_to_diff_scope(&mut self, variable_scopes: &VariableScopes<'a>) {
    let target = self.find_first_different_variable_scope(variable_scopes);
    self.refer_to_scope(target);
  }

  fn refer_to_scope(&mut self, target: usize) {
    let scopes = self.scope_context.variable_scopes[target..].to_vec();
    for scope in scopes {
      if let Some(dep) = scope.dep.clone() {
        self.consume(dep);
      }
    }
    self.consume(self.call_scope().get_exec_dep());
  }
}
