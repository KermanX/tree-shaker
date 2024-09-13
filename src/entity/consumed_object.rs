use super::{entity::Entity, unknown::UnknownEntity};
use crate::analyzer::Analyzer;

pub fn get_property<'a>(analyzer: &mut Analyzer<'a>, key: &Entity<'a>) -> (bool, Entity<'a>) {
  analyzer.may_throw();
  key.get_to_property_key().consume_self(analyzer);
  (true, UnknownEntity::new_unknown())
}

pub fn set_property<'a>(analyzer: &mut Analyzer<'a>, key: &Entity<'a>, value: Entity<'a>) -> bool {
  analyzer.may_throw();
  key.get_to_property_key().consume_self(analyzer);
  value.consume_as_unknown(analyzer);
  true
}

pub fn enumerate_properties<'a>(
  analyzer: &mut Analyzer<'a>,
) -> (bool, Vec<(bool, Entity<'a>, Entity<'a>)>) {
  analyzer.may_throw();
  UnknownEntity::new_unknown_to_entries_result(vec![])
}

pub fn delete_property<'a>(analyzer: &mut Analyzer<'a>, key: &Entity<'a>) -> bool {
  key.get_to_property_key().consume_self(analyzer);
  true
}

pub fn call<'a>(
  analyzer: &mut Analyzer<'a>,
  this: &Entity<'a>,
  args: &Entity<'a>,
) -> (bool, Entity<'a>) {
  analyzer.may_throw();
  this.consume_as_unknown(analyzer);
  args.consume_as_unknown(analyzer);
  (true, UnknownEntity::new_unknown())
}

pub fn r#await<'a>(analyzer: &mut Analyzer<'a>) -> (bool, Entity<'a>) {
  analyzer.may_throw();
  (true, UnknownEntity::new_unknown())
}

pub fn iterate<'a>(analyzer: &mut Analyzer<'a>) -> (bool, Option<Entity<'a>>) {
  analyzer.may_throw();
  (true, Some(UnknownEntity::new_unknown()))
}

pub fn get_to_string<'a>() -> Entity<'a> {
  UnknownEntity::new_unknown()
}

pub fn get_to_array<'a>(length: usize) -> (Vec<Entity<'a>>, Entity<'a>) {
  UnknownEntity::new_unknown_to_array_result(length, vec![])
}
