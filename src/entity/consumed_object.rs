use super::Entity;
use crate::{
  analyzer::Analyzer,
  consumable::{box_consumable, Consumable},
};

pub fn get_property<'a>(
  rc: Entity<'a>,
  analyzer: &mut Analyzer<'a>,
  dep: Consumable<'a>,
  key: Entity<'a>,
) -> Entity<'a> {
  if analyzer.config.unknown_property_read_side_effects {
    analyzer.may_throw();
    analyzer.consume(dep);
    analyzer.refer_global();
    key.consume(analyzer);
    analyzer.factory.unknown
  } else {
    analyzer.factory.new_computed_unknown(box_consumable((rc.clone(), dep, key.clone())))
  }
}

pub fn set_property<'a>(
  analyzer: &mut Analyzer<'a>,
  dep: Consumable<'a>,
  key: Entity<'a>,
  value: Entity<'a>,
) {
  analyzer.may_throw();
  analyzer.consume(dep);
  analyzer.refer_global();
  key.get_to_property_key(analyzer).consume(analyzer);
  value.consume(analyzer);
}

pub fn enumerate_properties<'a>(
  rc: Entity<'a>,
  analyzer: &mut Analyzer<'a>,
  dep: Consumable<'a>,
) -> Vec<(bool, Entity<'a>, Entity<'a>)> {
  if analyzer.config.unknown_property_read_side_effects {
    analyzer.may_throw();
    analyzer.consume(dep);
    analyzer.refer_global();
    vec![(false, analyzer.factory.unknown, analyzer.factory.unknown)]
  } else {
    let unknown = analyzer.factory.new_computed_unknown(box_consumable((rc.clone(), dep)));
    vec![(false, unknown.clone(), unknown)]
  }
}

pub fn delete_property<'a>(analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, key: Entity<'a>) {
  analyzer.consume(dep);
  key.get_to_property_key(analyzer).consume(analyzer);
}

pub fn call<'a>(
  analyzer: &mut Analyzer<'a>,
  dep: Consumable<'a>,
  this: Entity<'a>,
  args: Entity<'a>,
) -> Entity<'a> {
  analyzer.may_throw();
  analyzer.consume(dep);
  analyzer.refer_global();
  this.consume(analyzer);
  args.consume(analyzer);
  analyzer.factory.unknown
}

pub fn r#await<'a>(analyzer: &mut Analyzer<'a>, dep: Consumable<'a>) -> Entity<'a> {
  analyzer.may_throw();
  analyzer.consume(dep);
  analyzer.refer_global();
  analyzer.factory.unknown
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
  (vec![], Some(analyzer.factory.unknown))
}

pub fn get_to_string<'a>(analyzer: &Analyzer<'a>) -> Entity<'a> {
  analyzer.factory.unknown_string
}

pub fn get_to_numeric<'a>(analyzer: &Analyzer<'a>) -> Entity<'a> {
  // Possibly number or bigint
  analyzer.factory.unknown
}
