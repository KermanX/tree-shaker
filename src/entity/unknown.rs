use super::entity::{Entity, EntityTrait};
use crate::analyzer::Analyzer;
use std::rc::Rc;

#[derive(Debug)]
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

#[derive(Debug)]
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
}

impl<'a> UnknownEntity<'a> {
  pub fn new_unknown() -> Entity<'a> {
    Rc::new(Self { kind: UnknownEntityKind::Unknown, deps: Vec::new() })
  }
}
