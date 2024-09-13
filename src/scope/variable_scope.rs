use crate::ast::DeclarationKind;
use crate::entity::entity::Entity;
use crate::entity::literal::LiteralEntity;
use crate::entity::unknown::UnknownEntity;
use oxc::semantic::ScopeId;
use oxc::semantic::SymbolId;
use rustc_hash::FxHashMap;
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Debug)]
pub struct VariableScope<'a> {
  pub id: ScopeId,
  pub has_effect: bool,
  /// (is_consumed_exhaustively, entity)
  pub variables: FxHashMap<SymbolId, (bool, Option<Entity<'a>>)>,
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

  pub fn declare(&mut self, kind: DeclarationKind, symbol: SymbolId, value: Option<Entity<'a>>) {
    if kind.is_var() {
      let old = self.variables.get(&symbol);
      let new = match old {
        Some(old @ (true, _)) => old.clone(),
        _ => (false, Some(value.unwrap_or_else(LiteralEntity::new_undefined))),
      };
      self.variables.insert(symbol, new);
    } else {
      let old = self.variables.insert(symbol, (false, value));
      if old.is_some() && !kind.allow_override_var() {
        // TODO: error "Variable already declared"
      }
    }
  }

  pub fn init(&mut self, symbol: SymbolId, value: Entity<'a>) {
    let old = self.variables.get_mut(&symbol).unwrap();
    old.1 = Some(value);
  }

  pub fn read(&self, symbol: &SymbolId) -> (bool, Entity<'a>) {
    let (consumed, value) = self.variables.get(symbol).unwrap();
    let value = value.as_ref().map_or_else(
      || {
        // TODO: throw TDZ error
        UnknownEntity::new_unknown()
      },
      Entity::clone,
    );
    (*consumed, value)
  }

  pub fn write(&mut self, symbol: SymbolId, (consumed, value): (bool, Entity<'a>)) {
    let old = self.variables.get_mut(&symbol).unwrap();
    if old.1.is_none() {
      // TODO: throw TDZ error
    }
    *old = (consumed, Some(value));
  }
}
