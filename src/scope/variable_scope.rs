use crate::ast::DeclarationKind;
use crate::entity::entity::Entity;
use oxc::semantic::ScopeId;
use oxc::semantic::SymbolId;
use rustc_hash::FxHashMap;
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Debug)]
pub struct VariableScope<'a> {
  pub id: ScopeId,
  pub has_effect: bool,
  /// (is_consumed_exhaustively, entity)
  pub variables: FxHashMap<SymbolId, (bool, Entity<'a>)>,
  pub cf_scope_index: usize,
}

static VARIABLE_SCOPE_ID: AtomicU32 = AtomicU32::new(0);

impl<'a> VariableScope<'a> {
  pub fn new(cf_scope_index: usize) -> Self {
    Self {
      id: ScopeId::new(VARIABLE_SCOPE_ID.fetch_add(1, Ordering::Relaxed)),
      has_effect: false,
      variables: FxHashMap::default(),
      cf_scope_index,
    }
  }

  pub fn set(
    &mut self,
    symbol: SymbolId,
    entity: (bool, Entity<'a>),
  ) -> Option<(bool, Entity<'a>)> {
    self.variables.insert(symbol, entity)
  }

  pub fn declare(&mut self, kind: DeclarationKind, symbol: SymbolId, entity: Entity<'a>) {
    if kind.is_var() {
      let old = self.get(&symbol);
      let new = match old {
        Some(old @ (true, _)) => old.clone(),
        _ => (false, entity),
      };
      self.set(symbol, new);
    } else {
      let old = self.set(symbol, (false, entity));
      if old.is_some() && !kind.allow_override_var() {
        // TODO: error "Variable already declared"
      }
    }
  }

  pub fn get(&self, symbol: &SymbolId) -> Option<&(bool, Entity<'a>)> {
    self.variables.get(symbol)
  }

  pub fn has(&self, symbol: &SymbolId) -> bool {
    self.variables.contains_key(symbol)
  }
}
