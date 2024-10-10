use super::{Entity, UnknownEntity};
use crate::{
  analyzer::Analyzer,
  consumable::{box_consumable, Consumable},
};

pub fn get_property<'a>(
  rc: &Entity<'a>,
  analyzer: &mut Analyzer<'a>,
  dep: Consumable<'a>,
  key: &Entity<'a>,
) -> Entity<'a> {
  if analyzer.config.unknown_property_read_side_effects {
    analyzer.may_throw();
    analyzer.consume(dep);
    analyzer.refer_global();
    key.consume(analyzer);
    UnknownEntity::new_unknown()
  } else {
    UnknownEntity::new_computed_unknown(box_consumable((rc.clone(), dep, key.clone())))
  }
}

pub fn set_property<'a>(
  analyzer: &mut Analyzer<'a>,
  dep: Consumable<'a>,
  key: &Entity<'a>,
  value: Entity<'a>,
) {
  analyzer.may_throw();
  analyzer.consume(dep);
  analyzer.refer_global();
  key.get_to_property_key().consume(analyzer);
  value.consume(analyzer);
}

pub fn enumerate_properties<'a>(
  rc: &Entity<'a>,
  analyzer: &mut Analyzer<'a>,
  dep: Consumable<'a>,
) -> Vec<(bool, Entity<'a>, Entity<'a>)> {
  if analyzer.config.unknown_property_read_side_effects {
    analyzer.may_throw();
    analyzer.consume(dep);
    analyzer.refer_global();
    vec![(false, UnknownEntity::new_unknown(), UnknownEntity::new_unknown())]
  } else {
    let unknown = UnknownEntity::new_computed_unknown(box_consumable((rc.clone(), dep)));
    vec![(false, unknown.clone(), unknown)]
  }
}

pub fn delete_property<'a>(analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, key: &Entity<'a>) {
  analyzer.consume(dep);
  key.get_to_property_key().consume(analyzer);
}

pub fn call<'a>(
  analyzer: &mut Analyzer<'a>,
  dep: Consumable<'a>,
  this: &Entity<'a>,
  args: &Entity<'a>,
) -> Entity<'a> {
  analyzer.may_throw();
  analyzer.consume(dep);
  analyzer.refer_global();
  this.consume(analyzer);
  args.consume(analyzer);
  UnknownEntity::new_unknown()
}

pub fn r#await<'a>(analyzer: &mut Analyzer<'a>, dep: Consumable<'a>) -> Entity<'a> {
  analyzer.may_throw();
  analyzer.consume(dep);
  analyzer.refer_global();
  UnknownEntity::new_unknown()
}

pub fn iterate<'a>(
  analyzer: &mut Analyzer<'a>,
  dep: Consumable<'a>,
) -> (Vec<Entity<'a>>, Option<Entity<'a>>) {
  analyzer.may_throw();
  if analyzer.config.iterate_side_effects {
    analyzer.consume(dep);
    analyzer.refer_global();
  }
  (vec![], Some(UnknownEntity::new_unknown()))
}

pub fn get_to_string<'a>() -> Entity<'a> {
  UnknownEntity::new_string()
}

pub fn get_to_numeric<'a>() -> Entity<'a> {
  // Possibly number or bigint
  UnknownEntity::new_unknown()
}
