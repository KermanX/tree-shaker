use super::{
  entity::{EnumeratedProperties, IteratedElements},
  Entity,
};
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
  let dep = (rc, dep, key);
  if analyzer.is_inside_pure() || !analyzer.config.unknown_property_read_side_effects {
    analyzer.factory.computed_unknown(dep)
  } else {
    analyzer.may_throw();
    analyzer.consume(dep);
    analyzer.refer_to_global();
    analyzer.factory.unknown()
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
  analyzer.refer_to_global();
  key.get_to_property_key(analyzer).consume(analyzer);
  value.consume(analyzer);
}

pub fn enumerate_properties<'a>(
  rc: Entity<'a>,
  analyzer: &mut Analyzer<'a>,
  dep: Consumable<'a>,
) -> EnumeratedProperties<'a> {
  if analyzer.config.unknown_property_read_side_effects {
    analyzer.may_throw();
    analyzer.consume(dep);
    analyzer.refer_to_global();
    (vec![(false, analyzer.factory.unknown(), analyzer.factory.unknown())], box_consumable(()))
  } else {
    (
      vec![(false, analyzer.factory.unknown(), analyzer.factory.unknown())],
      box_consumable((rc.clone(), dep)),
    )
  }
}

pub fn delete_property<'a>(analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, key: Entity<'a>) {
  analyzer.consume(dep);
  analyzer.refer_to_global();
  key.get_to_property_key(analyzer).consume(analyzer);
}

pub fn call<'a>(
  rc: Entity<'a>,
  analyzer: &mut Analyzer<'a>,
  dep: Consumable<'a>,
  this: Entity<'a>,
  args: Entity<'a>,
) -> Entity<'a> {
  let dep = (rc, dep, this, args);
  if analyzer.is_inside_pure() {
    analyzer.factory.computed_unknown(dep)
  } else {
    analyzer.consume(dep);
    analyzer.may_throw();
    analyzer.refer_to_global();
    analyzer.factory.unknown()
  }
}

pub fn construct<'a>(
  rc: Entity<'a>,
  analyzer: &mut Analyzer<'a>,
  dep: Consumable<'a>,
  args: Entity<'a>,
) -> Entity<'a> {
  let dep = (rc, dep, args);
  if analyzer.is_inside_pure() {
    analyzer.factory.computed_unknown(dep)
  } else {
    analyzer.consume(dep);
    analyzer.may_throw();
    analyzer.refer_to_global();
    analyzer.factory.unknown()
  }
}

pub fn r#await<'a>(analyzer: &mut Analyzer<'a>, dep: Consumable<'a>) -> Entity<'a> {
  analyzer.may_throw();
  analyzer.consume(dep);
  analyzer.refer_to_global();
  analyzer.factory.unknown()
}

pub fn iterate<'a>(analyzer: &mut Analyzer<'a>, dep: Consumable<'a>) -> IteratedElements<'a> {
  analyzer.may_throw();
  if analyzer.config.iterate_side_effects {
    analyzer.consume(dep);
    analyzer.refer_to_global();
  }
  (vec![], Some(analyzer.factory.unknown()), box_consumable(()))
}

pub fn get_to_string<'a>(analyzer: &Analyzer<'a>) -> Entity<'a> {
  analyzer.factory.unknown_string
}

pub fn get_to_numeric<'a>(analyzer: &Analyzer<'a>) -> Entity<'a> {
  // Possibly number or bigint
  analyzer.factory.unknown()
}
