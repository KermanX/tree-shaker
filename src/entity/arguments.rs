use super::entity::{Entity, EntityTrait};
use std::rc::Rc;

#[derive(Debug)]
pub(crate) struct ArgumentsEntity<'a> {
  pub(crate) arguments: Vec<(bool, Entity<'a>)>,
}

impl<'a> EntityTrait<'a> for ArgumentsEntity<'a> {
  fn consume_self(&self, _analyzer: &mut crate::analyzer::Analyzer<'a>) {
    unreachable!()
  }

  fn consume_as_unknown(&self, _analyzer: &mut crate::analyzer::Analyzer<'a>) {
    unreachable!()
  }

  fn consume_as_array(
    &self,
    analyzer: &mut crate::analyzer::Analyzer<'a>,
    length: usize,
  ) -> (Vec<Entity<'a>>, Entity<'a>) {
    todo!("p4")
  }

  fn get_property(&self, _key: &Entity<'a>) -> Entity<'a> {
    unreachable!()
  }
}

impl<'a> ArgumentsEntity<'a> {
  pub(crate) fn new(arguments: Vec<(bool, Entity<'a>)>) -> Entity<'a> {
    Rc::new(Self { arguments })
  }
}
