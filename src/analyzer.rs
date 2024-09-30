use crate::{
  ast::{AstType2, DeclarationKind},
  builtins::Builtins,
  data::{get_node_ptr, Diagnostics, ExtraData, ReferredNodes, StatementVecData, VarDeclarations},
  entity::{
    Consumable, Entity, EntityOpHost, ForwardedEntity, LabelEntity, LiteralEntity, UnionEntity,
    UnknownEntity,
  },
  scope::{exhaustive::TrackerRunner, variable_scope::VariableScopes, ScopeContext},
  TreeShakeConfig,
};
use oxc::{
  allocator::Allocator,
  ast::ast::Program,
  semantic::{Semantic, SymbolId},
  span::GetSpan,
};
use rustc_hash::{FxHashMap, FxHashSet};
use std::mem;

pub struct Analyzer<'a> {
  pub config: TreeShakeConfig,
  pub allocator: &'a Allocator,
  pub semantic: Semantic<'a>,
  pub diagnostics: &'a mut Diagnostics,
  pub data: ExtraData<'a>,
  pub referred_nodes: ReferredNodes<'a>,
  pub var_decls: VarDeclarations<'a>,
  pub named_exports: Vec<SymbolId>,
  pub default_export: Option<Entity<'a>>,
  pub symbol_decls: FxHashMap<SymbolId, (DeclarationKind, VariableScopes<'a>, Consumable<'a>)>,
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
      data: Default::default(),
      referred_nodes: Default::default(),
      var_decls: Default::default(),
      named_exports: Vec::new(),
      default_export: None,
      symbol_decls: Default::default(),
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
      let entity = self.read_symbol(&symbol).unwrap();
      entity.consume(self);

      let (_, _, decl_dep) = self.symbol_decls.get(&symbol).unwrap();
      self.consume(decl_dep.clone());
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
    self.diagnostics.insert(message.into());
  }
}

impl<'a> Analyzer<'a> {
  pub fn declare_symbol(
    &mut self,
    symbol: SymbolId,
    decl_dep: impl Into<Consumable<'a>>,
    exporting: bool,
    kind: DeclarationKind,
    value: Option<Entity<'a>>,
  ) {
    let old_decl = self.symbol_decls.get(&symbol);
    if matches!(old_decl, Some((kind,_,_)) if kind.is_untracked()) {
      self.consume(decl_dep.into());
      value.map(|val| val.consume(self));
      return;
    }
    let old_decl_dep = old_decl.map(|(_, _, decl_dep)| decl_dep.clone());

    if exporting {
      self.named_exports.push(symbol);
    }
    if kind == DeclarationKind::FunctionParameter {
      self.call_scope_mut().args.1.push(symbol);
    }
    if kind == DeclarationKind::Var {
      self.insert_var_decl(symbol);
    }

    let variable_scopes = if kind.is_var() {
      let index = self.call_scope().variable_scope_index;
      self.scope_context.variable_scopes[..index + 1].to_vec()
    } else {
      self.scope_context.variable_scopes.clone()
    };
    variable_scopes.last().unwrap().declare(self, kind, symbol, value);
    let decl_dep = if let Some(old_decl_dep) = old_decl_dep {
      (old_decl_dep, decl_dep.into()).into()
    } else {
      decl_dep.into()
    };
    self.symbol_decls.insert(symbol, (kind, variable_scopes, decl_dep));
  }

  pub fn init_symbol(&mut self, symbol: SymbolId, init: Option<Entity<'a>>, dep: Consumable<'a>) {
    let (kind, variable_scopes, _) = &self.symbol_decls.get(&symbol).unwrap();
    let init = if kind.is_redeclarable() {
      init
    } else {
      Some(init.unwrap_or_else(LiteralEntity::new_undefined))
    };
    if let Some(init) = init {
      let value = ForwardedEntity::new(init, dep);
      if kind.is_untracked() {
        value.consume(self);
      } else if kind.is_var() {
        self.write_symbol(&symbol, value);
      } else {
        let variable_scope = variable_scopes.last().unwrap().clone();
        variable_scope.init(self, symbol, value);
      }
    }
  }

  /// `None` for TDZ
  pub fn read_symbol(&mut self, symbol: &SymbolId) -> Option<Entity<'a>> {
    if let Some((kind, variable_scopes, decl_dep)) = self.symbol_decls.get(symbol) {
      if kind.is_untracked() {
        return Some(UnknownEntity::new_unknown());
      }
      let decl_dep = decl_dep.clone();
      let variable_scope = variable_scopes.last().unwrap().clone();
      let target_cf_scope = self.find_first_different_cf_scope(&variable_scope.cf_scopes);
      let (_, val) = variable_scope.read(symbol);
      if let Some(val) = &val {
        self.mark_exhaustive_read(val, *symbol, target_cf_scope);
      } else {
        self.consume(decl_dep);
        self.handle_tdz(target_cf_scope);
      }
      val
    } else {
      self.on_unresolved_reference(*symbol)
    }
  }

  pub fn write_symbol(&mut self, symbol: &SymbolId, new_val: Entity<'a>) {
    if let Some((kind, variable_scopes, decl_dep)) = self.symbol_decls.get(symbol) {
      if kind.is_untracked() {
        new_val.consume(self);
        return;
      }
      let decl_variable_scope = variable_scopes.last().unwrap().clone();
      let variable_scope_cf_scopes = &decl_variable_scope.cf_scopes;
      let target_cf_scope = self.find_first_different_cf_scope(variable_scope_cf_scopes);
      let target_variable_scope = self.find_first_different_variable_scope(variable_scopes);
      let dep = self.get_assignment_deps(target_variable_scope, decl_dep.clone());
      let (has_been_consumed_exhaustively, old_val) = decl_variable_scope.read(symbol);
      if has_been_consumed_exhaustively {
        self.consume(dep);
        new_val.consume(self);
      } else {
        let should_consume = if let Some(old_val) = &old_val {
          if old_val.test_is_completely_unknown() {
            false
          } else {
            self.mark_exhaustive_write(*symbol, target_cf_scope)
          }
        } else {
          self.handle_tdz(target_cf_scope);
          true
        };
        let entity_to_set = if should_consume {
          self.consume(dep);
          old_val.map(|v| v.consume(self));
          new_val.consume(self);
          (true, UnknownEntity::new_unknown())
        } else {
          let indeterminate =
            self.is_relatively_indeterminate(target_cf_scope, variable_scope_cf_scopes);

          (
            false,
            ForwardedEntity::new(
              if indeterminate {
                UnionEntity::new(vec![old_val.unwrap(), new_val])
              } else {
                new_val
              },
              dep,
            ),
          )
        };
        decl_variable_scope.write(self, *symbol, entity_to_set);
        self.exec_exhaustive_deps(should_consume, *symbol);
      }
    } else {
      new_val.consume(self);
      self.on_unresolved_reference(*symbol);
    }
  }

  fn on_unresolved_reference(&mut self, symbol: SymbolId) -> Option<Entity<'a>> {
    self.symbol_decls.insert(symbol, (DeclarationKind::UntrackedVar, vec![], ().into()));
    if self.semantic.symbols().get_flags(symbol).is_function_scoped_declaration() {
      self.insert_var_decl(symbol);
      Some(UnknownEntity::new_unknown())
    } else {
      self.explicit_throw_unknown("Unresolved identifier reference");
      None
    }
  }

  fn insert_var_decl(&mut self, symbol: SymbolId) {
    let key = self.call_scope().source;
    self.var_decls.entry(key).or_default().insert(symbol);
  }

  fn handle_tdz(&mut self, target_cf_scope: usize) {
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
      let mut deps = self
        .scope_context
        .variable_scopes
        .iter()
        .filter_map(|scope| scope.dep.clone())
        .collect::<Vec<_>>();
      deps.push(self.call_scope().get_exec_dep());
      self.consume(deps);
    }
  }
}
