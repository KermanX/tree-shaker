use super::{
  consumed_object,
  entity::{EnumeratedProperties, IteratedElements},
  Entity, EntityFactory, EntityTrait, TypeofResult,
};
use crate::{
  analyzer::Analyzer,
  builtins::Prototype,
  consumable::{box_consumable, Consumable, ConsumableTrait},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveEntity {
  // TODO: NumericString, NoneEmptyString, ...
  Mixed,
  String,
  Number,
  BigInt,
  Boolean,
  Symbol,
}

impl<'a> EntityTrait<'a> for PrimitiveEntity {
  fn consume(&self, _analyzer: &mut Analyzer<'a>) {}

  fn get_property(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
  ) -> Entity<'a> {
    if *self == PrimitiveEntity::Mixed {
      analyzer.factory.computed_unknown((rc, dep, key))
    } else {
      let prototype = self.get_prototype(analyzer);
      prototype.get_property(analyzer, rc, key, dep)
    }
  }

  fn set_property(
    &self,
    _rc: Entity<'a>,
    _analyzer: &mut Analyzer<'a>,
    _dep: Consumable<'a>,
    _key: Entity<'a>,
    _value: Entity<'a>,
  ) {
    // No effect
  }

  fn enumerate_properties(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> EnumeratedProperties<'a> {
    if *self == PrimitiveEntity::String {
      (
        vec![(false, analyzer.factory.unknown_string, analyzer.factory.unknown_string)],
        box_consumable((rc.clone(), dep)),
      )
    } else {
      (vec![], box_consumable((rc.clone(), dep)))
    }
  }

  fn delete_property(&self, _analyzer: &mut Analyzer<'a>, _dep: Consumable<'a>, _key: Entity<'a>) {
    // No effect
  }

  fn call(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: Entity<'a>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    analyzer.thrown_builtin_error("Cannot call non-object");
    consumed_object::call(rc, analyzer, dep, this, args)
  }

  fn construct(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    analyzer.thrown_builtin_error("Cannot construct non-object");
    consumed_object::construct(rc, analyzer, dep, args)
  }

  fn r#await(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> Entity<'a> {
    analyzer.factory.computed(rc, dep)
  }

  fn iterate(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> IteratedElements<'a> {
    if *self == PrimitiveEntity::String {
      return (vec![], Some(analyzer.factory.unknown()), box_consumable((rc, dep)));
    }
    analyzer.thrown_builtin_error("Cannot iterate non-object");
    self.consume(analyzer);
    consumed_object::iterate(analyzer, dep)
  }

  fn get_destructable(&self, rc: Entity<'a>, dep: Consumable<'a>) -> Consumable<'a> {
    box_consumable((rc, dep))
  }

  fn get_typeof(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    if let Some(str) = self.test_typeof().to_string() {
      analyzer.factory.string(str)
    } else {
      analyzer.factory.unknown_string
    }
  }

  fn get_to_string(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    analyzer.factory.unknown_string
  }

  fn get_to_numeric(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    analyzer.factory.unknown()
  }

  fn get_to_boolean(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    match self.test_truthy() {
      Some(val) => analyzer.factory.boolean(val),
      None => analyzer.factory.unknown_boolean,
    }
  }

  fn get_to_property_key(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    analyzer.factory.unknown()
  }

  fn test_typeof(&self) -> TypeofResult {
    match self {
      PrimitiveEntity::String => TypeofResult::String,
      PrimitiveEntity::Number => TypeofResult::Number,
      PrimitiveEntity::BigInt => TypeofResult::BigInt,
      PrimitiveEntity::Boolean => TypeofResult::Boolean,
      PrimitiveEntity::Symbol => TypeofResult::Symbol,
      PrimitiveEntity::Mixed => TypeofResult::_Unknown,
    }
  }

  fn test_truthy(&self) -> Option<bool> {
    match self {
      PrimitiveEntity::Symbol => Some(true),
      _ => None,
    }
  }

  fn test_nullish(&self) -> Option<bool> {
    Some(false)
  }
}

impl<'a> PrimitiveEntity {
  fn get_prototype<'b>(&self, analyzer: &mut Analyzer<'a>) -> &'a Prototype<'a> {
    match self {
      PrimitiveEntity::String => &analyzer.builtins.prototypes.string,
      PrimitiveEntity::Number => &analyzer.builtins.prototypes.number,
      PrimitiveEntity::BigInt => &analyzer.builtins.prototypes.bigint,
      PrimitiveEntity::Boolean => &analyzer.builtins.prototypes.boolean,
      PrimitiveEntity::Symbol => &analyzer.builtins.prototypes.symbol,
      PrimitiveEntity::Mixed => unreachable!(),
    }
  }
}

macro_rules! unknown_entity_ctors {
  ($($name:ident -> $var:ident,)*) => {
    $(
      #[allow(unused)]
      pub fn $name<T: ConsumableTrait<'a> + 'a>(&self, dep: T) -> Entity<'a> {
        self.computed(self.$var, dep)
      }
    )*
  };
}

impl<'a> EntityFactory<'a> {
  unknown_entity_ctors! {
    computed_unknown_primitive -> unknown_primitive,
    computed_unknown_boolean -> unknown_boolean,
    computed_unknown_number -> unknown_number,
    computed_unknown_string -> unknown_string,
    computed_unknown_bigint -> unknown_bigint,
    computed_unknown_symbol -> unknown_symbol,
  }
}
