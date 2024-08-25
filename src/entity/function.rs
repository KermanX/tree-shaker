use super::{
  dep::EntityDep,
  entity::{Entity, EntityTrait},
  unknown::UnknownEntity,
};
use std::rc::Rc;

#[derive(Debug)]
pub(crate) struct FunctionEntity<'a> {
  pub(crate) source: EntityDep<'a>,
}

impl<'a> EntityTrait<'a> for FunctionEntity<'a> {
  fn consume_self(&self, analyzer: &mut crate::analyzer::Analyzer<'a>) {
    analyzer.refer_dep(self.source);
  }

  fn consume_as_unknown(&self, analyzer: &mut crate::analyzer::Analyzer<'a>) {
    self.call(analyzer, &UnknownEntity::new_unknown(), &UnknownEntity::new_unknown());
  }

  fn call(
    &self,
    analyzer: &mut crate::analyzer::Analyzer<'a>,
    this: &Entity<'a>,
    args: &Entity<'a>,
  ) -> Entity<'a> {
    self.consume_self(analyzer);
    match &self.source {
      EntityDep::Function(node) => analyzer.call_function(node, this.clone(), args.clone()),
      EntityDep::ArrowFunctionExpression(node) => todo!(),
      _ => UnknownEntity::new_unknown(),
    }
  }

  fn get_property(&self, _key: &Entity<'a>) -> Entity<'a> {
    todo!("built-ins")
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
