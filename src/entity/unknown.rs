use super::{
  entity::{Entity, EntityTrait},
  literal::LiteralEntity,
};
use crate::analyzer::Analyzer;
use std::rc::Rc;

#[derive(Debug, Clone, Copy)]
pub(crate) enum UnknownEntityKind {
  // TODO: NumericString, NoneEmptyString, ...
  String,
  Number,
  BigInt,
  Boolean,
  Symbol,
  Array,
  Function,
  Object,
  Unknown,
}

#[derive(Debug, Clone)]
pub(crate) struct UnknownEntity<'a> {
  pub kind: UnknownEntityKind,
  pub deps: Vec<Entity<'a>>,
}

impl<'a> EntityTrait<'a> for UnknownEntity<'a> {
  fn consume_self(&self, analyzer: &mut Analyzer<'a>) {
    for dep in &self.deps {
      dep.consume_self(analyzer);
    }
  }

  fn get_typeof(&self) -> Entity<'a> {
    LiteralEntity::new_string(match &self.kind {
      UnknownEntityKind::String => "string",
      UnknownEntityKind::Number => "number",
      UnknownEntityKind::BigInt => "bigint",
      UnknownEntityKind::Boolean => "boolean",
      UnknownEntityKind::Symbol => "symbol",
      UnknownEntityKind::Array => "object",
      UnknownEntityKind::Function => "function",
      UnknownEntityKind::Object => "object",
      UnknownEntityKind::Unknown => {
        return UnknownEntity::new(UnknownEntityKind::String, self.deps.clone())
      }
    })
  }

  fn get_property(&self, key: &Entity<'a>) -> Entity<'a> {
    // TODO: Builtin properties
    let mut deps = self.deps.clone();
    deps.push(key.clone());
    Rc::new(Self { kind: UnknownEntityKind::Unknown, deps })
  }

  fn test_truthy(&self) -> Option<bool> {
    match &self.kind {
      UnknownEntityKind::Symbol
      | UnknownEntityKind::Array
      | UnknownEntityKind::Function
      | UnknownEntityKind::Object => Some(true),
      _ => None,
    }
  }

  fn test_nullish(&self) -> Option<bool> {
    match &self.kind {
      UnknownEntityKind::Unknown => None,
      _ => Some(false),
    }
  }

  fn test_is_undefined(&self) -> Option<bool> {
    match &self.kind {
      UnknownEntityKind::Unknown => None,
      _ => Some(false),
    }
  }
}

impl<'a> UnknownEntity<'a> {
  pub fn new(kind: UnknownEntityKind, deps: Vec<Entity<'a>>) -> Entity<'a> {
    Rc::new(Self { kind, deps })
  }

  pub fn new_unknown() -> Entity<'a> {
    Rc::new(Self { kind: UnknownEntityKind::Unknown, deps: Vec::new() })
  }
}
