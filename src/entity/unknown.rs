use super::{
  consumed_object, entity::EnumeratedProperties, Entity, EntityFactory, EntityTrait, TypeofResult,
};
use crate::{
  analyzer::Analyzer,
  builtins::Prototype,
  consumable::{box_consumable, Consumable, ConsumableTrait},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnknownEntity {
  // TODO: NumericString, NoneEmptyString, ...
  String,
  Number,
  BigInt,
  Boolean,
  Symbol,
  Function,
  Regexp,
  Array,
  Object,
  Unknown,
}

impl<'a> EntityTrait<'a> for UnknownEntity {
  fn consume(&self, _analyzer: &mut Analyzer<'a>) {
    // FIXME: Should set self to UnknownEntity::Object here
  }

  fn get_property(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
  ) -> Entity<'a> {
    if self.maybe_object() {
      if analyzer.config.unknown_property_read_side_effects {
        self.consume(analyzer);
      }
      consumed_object::get_property(rc, analyzer, dep, key)
    } else {
      let prototype = self.get_prototype(analyzer);
      prototype.get_property(analyzer, rc, key, dep)
    }
  }

  fn set_property(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
    value: Entity<'a>,
  ) {
    if self.maybe_object() {
      self.consume(analyzer);
      consumed_object::set_property(analyzer, dep, key, value)
    } else {
      // Primitives. No effect
    }
  }

  fn enumerate_properties(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> EnumeratedProperties<'a> {
    if self.maybe_object() {
      if analyzer.config.unknown_property_read_side_effects {
        self.consume(analyzer);
      }
      consumed_object::enumerate_properties(rc, analyzer, dep)
    } else if *self == UnknownEntity::String {
      (
        vec![(false, analyzer.factory.unknown_string, analyzer.factory.unknown_string)],
        box_consumable((rc.clone(), dep)),
      )
    } else {
      (vec![], box_consumable((rc.clone(), dep)))
    }
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, key: Entity<'a>) {
    if self.maybe_object() {
      self.consume(analyzer);
      consumed_object::delete_property(analyzer, dep, key)
    } else {
      // No effect
    }
  }

  fn call(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: Entity<'a>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    if !self.maybe_object() {
      analyzer.thrown_builtin_error("Cannot call non-object");
    }
    self.consume(analyzer);
    consumed_object::call(analyzer, dep, this, args)
  }

  fn r#await(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> Entity<'a> {
    if self.maybe_object() {
      self.consume(analyzer);
      consumed_object::r#await(analyzer, dep)
    } else {
      analyzer.factory.computed(rc, dep)
    }
  }

  fn iterate(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> (Vec<Entity<'a>>, Option<Entity<'a>>) {
    if *self == UnknownEntity::String {
      return (vec![], Some(analyzer.factory.computed_unknown(rc)));
    }
    if !self.maybe_object() {
      analyzer.thrown_builtin_error("Cannot iterate non-object");
    }
    self.consume(analyzer);
    consumed_object::iterate(analyzer, dep)
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
    analyzer.factory.unknown
  }

  fn get_to_boolean(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    match self.test_truthy() {
      Some(val) => analyzer.factory.boolean(val),
      None => analyzer.factory.unknown_boolean,
    }
  }

  fn get_to_property_key(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    analyzer.factory.unknown
  }

  fn test_typeof(&self) -> TypeofResult {
    match self {
      UnknownEntity::String => TypeofResult::String,
      UnknownEntity::Number => TypeofResult::Number,
      UnknownEntity::BigInt => TypeofResult::BigInt,
      UnknownEntity::Boolean => TypeofResult::Boolean,
      UnknownEntity::Symbol => TypeofResult::Symbol,
      UnknownEntity::Function => TypeofResult::Function,
      UnknownEntity::Regexp => TypeofResult::Object,
      UnknownEntity::Array => TypeofResult::Object,
      UnknownEntity::Object => TypeofResult::Object | TypeofResult::Function,
      UnknownEntity::Unknown => TypeofResult::_Unknown,
    }
  }

  fn test_truthy(&self) -> Option<bool> {
    match self {
      UnknownEntity::Symbol
      | UnknownEntity::Function
      | UnknownEntity::Array
      | UnknownEntity::Object => Some(true),
      _ => None,
    }
  }

  fn test_nullish(&self) -> Option<bool> {
    match self {
      UnknownEntity::Unknown | UnknownEntity::Object => None,
      _ => Some(false),
    }
  }

  fn test_is_completely_unknown(&self) -> bool {
    matches!(self, UnknownEntity::Unknown)
  }
}

impl<'a> UnknownEntity {
  pub fn maybe_object(&self) -> bool {
    matches!(
      self,
      UnknownEntity::Object
        | UnknownEntity::Array
        | UnknownEntity::Function
        | UnknownEntity::Regexp
        | UnknownEntity::Unknown
    )
  }

  fn get_prototype<'b>(&self, analyzer: &mut Analyzer<'a>) -> &'a Prototype<'a> {
    match self {
      UnknownEntity::String => &analyzer.builtins.prototypes.string,
      UnknownEntity::Number => &analyzer.builtins.prototypes.number,
      UnknownEntity::BigInt => &analyzer.builtins.prototypes.bigint,
      UnknownEntity::Boolean => &analyzer.builtins.prototypes.boolean,
      UnknownEntity::Symbol => &analyzer.builtins.prototypes.symbol,
      UnknownEntity::Function => &analyzer.builtins.prototypes.function,
      UnknownEntity::Regexp => &analyzer.builtins.prototypes.regexp,
      UnknownEntity::Array => &analyzer.builtins.prototypes.array,
      UnknownEntity::Object => &analyzer.builtins.prototypes.object,
      UnknownEntity::Unknown => unreachable!(),
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
    computed_unknown -> unknown,
    computed_unknown_boolean -> unknown_boolean,
    computed_unknown_number -> unknown_number,
    computed_unknown_string -> unknown_string,
    computed_unknown_bigint -> unknown_bigint,
    computed_unknown_symbol -> unknown_symbol,
    computed_unknown_function -> unknown_function,
    computed_unknown_regexp -> unknown_regexp,
    computed_unknown_array -> unknown_array,
    computed_unknown_object -> unknown_object,
  }
}
