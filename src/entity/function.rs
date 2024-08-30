use super::{
  dep::{EntityDep, EntityDepNode},
  entity::{Entity, EntityTrait},
  forwarded::ForwardedEntity,
  literal::LiteralEntity,
  typeof_result::TypeofResult,
  unknown::{UnknownEntity, UnknownEntityKind},
};
use crate::analyzer::Analyzer;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub(crate) struct FunctionEntity<'a> {
  pub(crate) source: EntityDep<'a>,
}

impl<'a> EntityTrait<'a> for FunctionEntity<'a> {
  fn consume_self(&self, analyzer: &mut Analyzer<'a>) {
    analyzer.refer_dep(&self.source);
  }

  fn consume_as_unknown(&self, analyzer: &mut Analyzer<'a>) {
    self.consume_self(analyzer);
    let (_, ret_val) =
      self.call(analyzer, &UnknownEntity::new_unknown(), &UnknownEntity::new_unknown());
    ret_val.consume_as_unknown(analyzer);
  }

  fn get_property(&self, analyzer: &mut Analyzer<'a>, _key: &Entity<'a>) -> (bool, Entity<'a>) {
    todo!("built-ins")
  }

  fn set_property(
    &self,
    analyzer: &mut Analyzer<'a>,
    _key: &Entity<'a>,
    _value: Entity<'a>,
  ) -> bool {
    todo!("built-ins")
  }

  fn call(
    &self,
    analyzer: &mut Analyzer<'a>,
    this: &Entity<'a>,
    args: &Entity<'a>,
  ) -> (bool, Entity<'a>) {
    match &self.source.node {
      EntityDepNode::Function(node) => {
        let (has_effect, ret_val) = analyzer.call_function(node, this.clone(), args.clone());
        if has_effect {
          self.consume_self(analyzer);
        }
        (has_effect, ForwardedEntity::new(ret_val, self.source.clone()))
      }
      EntityDepNode::ArrowFunctionExpression(node) => todo!(),
      _ => unreachable!(),
    }
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

impl<'a> FunctionEntity<'a> {
  pub(crate) fn new(source: EntityDep<'a>) -> Entity<'a> {
    Rc::new(Self { source })
  }
}
