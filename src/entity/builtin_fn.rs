use super::{
  entity::{Entity, EntityTrait},
  literal::LiteralEntity,
  typeof_result::TypeofResult,
  unknown::{UnknownEntity, UnknownEntityKind},
};
use crate::analyzer::Analyzer;
use std::rc::Rc;

pub(crate) type BuiltinFnImplementation<'a> =
  fn(&mut Analyzer<'a>, &Entity<'a>, &Entity<'a>) -> (bool, Entity<'a>);

#[derive(Debug, Clone)]
pub(crate) struct BuiltinFnEntity<'a> {
  implementation: BuiltinFnImplementation<'a>,
}

impl<'a> EntityTrait<'a> for BuiltinFnEntity<'a> {
  fn consume_self(&self, _analyzer: &mut Analyzer<'a>) {}

  fn consume_as_unknown(&self, analyzer: &mut Analyzer<'a>) {
    self.consume_self(analyzer);
    let (_, ret_val) =
      self.call(analyzer, &UnknownEntity::new_unknown(), &UnknownEntity::new_unknown());
    ret_val.consume_as_unknown(analyzer);
  }

  fn get_property(&self, analyzer: &mut Analyzer<'a>, _key: &Entity<'a>) -> (bool, Entity<'a>) {
    todo!("built-ins & extra properties")
  }

  fn set_property(
    &self,
    analyzer: &mut Analyzer<'a>,
    _key: &Entity<'a>,
    _value: Entity<'a>,
  ) -> bool {
    todo!("built-ins & extra properties")
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, key: &Entity<'a>) -> bool {
    self.consume_as_unknown(analyzer);
    key.consume_self(analyzer);
    true
  }

  fn enumerate_properties(
    &self,
    analyzer: &mut Analyzer<'a>,
  ) -> (bool, Vec<(bool, Entity<'a>, Entity<'a>)>) {
    self.consume_as_unknown(analyzer);
    UnknownEntity::new_unknown_to_entries_result(vec![])
  }

  fn call(
    &self,
    analyzer: &mut Analyzer<'a>,
    this: &Entity<'a>,
    args: &Entity<'a>,
  ) -> (bool, Entity<'a>) {
    let (has_effect, ret_val) = (self.implementation)(analyzer, this, args);
    if has_effect {
      self.consume_self(analyzer);
    }
    (has_effect, ret_val)
  }

  fn r#await(&self, analyzer: &mut Analyzer<'a>) -> (bool, Entity<'a>) {
    (false, Rc::new(self.clone()))
  }

  fn get_typeof(&self) -> Entity<'a> {
    LiteralEntity::new_string("function")
  }

  fn get_to_string(&self) -> Entity<'a> {
    // FIXME: No Rc::new + clone
    UnknownEntity::new_with_deps(UnknownEntityKind::String, vec![Rc::new(self.clone())])
  }

  fn get_to_property_key(&self) -> Entity<'a> {
    self.get_to_string()
  }

  fn get_to_array(&self, length: usize) -> (Vec<Entity<'a>>, Entity<'a>) {
    UnknownEntity::new_unknown_to_array_result(length, vec![Rc::new(self.clone())])
  }

  fn test_typeof(&self) -> TypeofResult {
    TypeofResult::Function
  }

  fn test_truthy(&self) -> Option<bool> {
    Some(true)
  }

  fn test_nullish(&self) -> Option<bool> {
    Some(false)
  }
}

impl<'a> BuiltinFnEntity<'a> {
  pub(crate) fn new(implementation: BuiltinFnImplementation<'a>) -> Entity<'a> {
    Rc::new(Self { implementation })
  }
}
