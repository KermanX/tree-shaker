use super::{dep::EntityDep, entity::Entity, unknown::UnknownEntity};
use crate::analyzer::Analyzer;

pub fn get_property<'a>(
  analyzer: &mut Analyzer<'a>,
  dep: EntityDep,
  key: &Entity<'a>,
) -> Entity<'a> {
  analyzer.may_throw();
  analyzer.refer_dep(dep);
  analyzer.refer_global();
  key.get_to_property_key().consume_self(analyzer);
  UnknownEntity::new_unknown()
}

pub fn set_property<'a>(
  analyzer: &mut Analyzer<'a>,
  dep: EntityDep,
  key: &Entity<'a>,
  value: Entity<'a>,
) {
  analyzer.may_throw();
  analyzer.refer_dep(dep);
  analyzer.refer_global();
  key.get_to_property_key().consume_self(analyzer);
  value.consume_as_unknown(analyzer);
}

pub fn enumerate_properties<'a>(
  analyzer: &mut Analyzer<'a>,
  dep: EntityDep,
) -> Vec<(bool, Entity<'a>, Entity<'a>)> {
  analyzer.may_throw();
  analyzer.refer_dep(dep);
  analyzer.refer_global();
  vec![(false, UnknownEntity::new_unknown(), UnknownEntity::new_unknown())]
}

pub fn delete_property<'a>(analyzer: &mut Analyzer<'a>, dep: EntityDep, key: &Entity<'a>) {
  analyzer.refer_dep(dep);
  key.get_to_property_key().consume_self(analyzer);
}

pub fn call<'a>(
  analyzer: &mut Analyzer<'a>,
  dep: EntityDep,
  this: &Entity<'a>,
  args: &Entity<'a>,
) -> Entity<'a> {
  analyzer.may_throw();
  analyzer.refer_dep(dep);
  analyzer.refer_global();
  this.consume_as_unknown(analyzer);
  args.consume_as_unknown(analyzer);
  UnknownEntity::new_unknown()
}

pub fn r#await<'a>(analyzer: &mut Analyzer<'a>) -> (bool, Entity<'a>) {
  analyzer.may_throw();
  analyzer.refer_global();
  (true, UnknownEntity::new_unknown())
}

pub fn iterate<'a>(
  analyzer: &mut Analyzer<'a>,
  dep: EntityDep,
) -> (Vec<Entity<'a>>, Option<Entity<'a>>) {
  analyzer.may_throw();
  if analyzer.config.iterate_side_effects {
    analyzer.refer_dep(dep);
    analyzer.refer_global();
  }
  (vec![], Some(UnknownEntity::new_unknown()))
}

pub fn get_to_string<'a>() -> Entity<'a> {
  UnknownEntity::new_unknown()
}
