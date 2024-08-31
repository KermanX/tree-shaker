use super::{literal::LiteralEntity, typeof_result::TypeofResult, unknown::UnknownEntity};
use crate::analyzer::Analyzer;
use rustc_hash::FxHashSet;
use std::{fmt::Debug, rc::Rc};

pub(crate) trait EntityTrait<'a>: Debug {
  fn consume_self(&self, analyzer: &mut Analyzer<'a>);
  fn consume_as_unknown(&self, analyzer: &mut Analyzer<'a>);

  fn get_property(&self, analyzer: &mut Analyzer<'a>, key: &Entity<'a>) -> (bool, Entity<'a>);
  fn set_property(&self, analyzer: &mut Analyzer<'a>, key: &Entity<'a>, value: Entity<'a>) -> bool;
  fn enumerate_properties(
    &self,
    analyzer: &mut Analyzer<'a>,
  ) -> (bool, Vec<(bool, Entity<'a>, Entity<'a>)>);
  fn delete_property(&self, analyzer: &mut Analyzer<'a>, key: &Entity<'a>) -> bool;
  fn call(
    &self,
    analyzer: &mut Analyzer<'a>,
    this: &Entity<'a>,
    args: &Entity<'a>,
  ) -> (bool, Entity<'a>) {
    this.consume_as_unknown(analyzer);
    args.consume_as_unknown(analyzer);
    (true, UnknownEntity::new_unknown())
  }

  fn get_typeof(&self) -> Entity<'a>;
  fn get_to_string(&self) -> Entity<'a>;
  fn get_to_property_key(&self) -> Entity<'a>;
  fn get_to_array(&self, length: usize) -> (Vec<Entity<'a>>, Entity<'a>);
  fn get_to_literals(&self) -> Option<FxHashSet<LiteralEntity<'a>>> {
    None
  }
  fn get_literal(&self) -> Option<LiteralEntity<'a>> {
    self.get_to_literals().and_then(
      |set| {
        if set.len() == 1 {
          set.into_iter().next()
        } else {
          None
        }
      },
    )
  }

  fn test_typeof(&self) -> TypeofResult;
  fn test_truthy(&self) -> Option<bool>;
  fn test_nullish(&self) -> Option<bool>;
  fn test_is_undefined(&self) -> Option<bool> {
    let t = self.test_typeof();
    match (t == TypeofResult::Undefined, t.contains(TypeofResult::Undefined)) {
      (true, _) => Some(true),
      (false, true) => None,
      (false, false) => Some(false),
    }
  }
  fn test_is_completely_unknown(&self) -> bool {
    false
  }
}

pub(crate) type Entity<'a> = Rc<dyn EntityTrait<'a> + 'a>;
