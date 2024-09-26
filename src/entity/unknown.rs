use super::{
  computed::ComputedEntity,
  consumable::Consumable,
  consumed_object,
  entity::{Entity, EntityTrait},
  interactions::InteractionKind,
  literal::LiteralEntity,
  typeof_result::TypeofResult,
};
use crate::{analyzer::Analyzer, builtins::Prototype};

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
  fn consume(&self, _analyzer: &mut Analyzer<'a>) {}

  fn interact(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, kind: InteractionKind) {
    self.consume(analyzer);
    consumed_object::interact(analyzer, dep, kind)
  }

  fn get_property(
    &self,
    rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: &Entity<'a>,
  ) -> Entity<'a> {
    if self.maybe_object() {
      self.consume(analyzer);
      consumed_object::get_property(analyzer, dep, key)
    } else {
      let prototype = self.get_prototype(analyzer);
      prototype.get_property(rc, key, dep)
    }
  }

  fn set_property(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: &Entity<'a>,
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
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> Vec<(bool, Entity<'a>, Entity<'a>)> {
    if self.maybe_object() {
      self.consume(analyzer);
      consumed_object::enumerate_properties(analyzer, dep)
    } else {
      vec![]
    }
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, key: &Entity<'a>) {
    if self.maybe_object() {
      consumed_object::delete_property(analyzer, dep, key)
    } else {
      // No effect
    }
  }

  fn call(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: &Entity<'a>,
    args: &Entity<'a>,
  ) -> Entity<'a> {
    if !self.maybe_object() {
      // TODO: throw warning
      analyzer.explicit_throw_unknown();
    }
    self.consume(analyzer);
    consumed_object::call(analyzer, dep, this, args)
  }

  fn r#await(&self, rc: &Entity<'a>, analyzer: &mut Analyzer<'a>) -> Entity<'a> {
    if self.maybe_object() {
      self.consume(analyzer);
      UnknownEntity::new_unknown()
    } else {
      rc.clone()
    }
  }

  fn iterate(
    &self,
    rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> (Vec<Entity<'a>>, Option<Entity<'a>>) {
    if *self == UnknownEntity::Array {
      return (vec![], Some(UnknownEntity::new_computed_unknown(rc.clone())));
    }
    if !self.maybe_object() {
      // TODO: throw warning
      analyzer.explicit_throw_unknown();
    }
    self.consume(analyzer);
    consumed_object::iterate(analyzer, dep)
  }

  fn get_typeof(&self) -> Entity<'a> {
    if let Some(str) = self.test_typeof().to_string() {
      LiteralEntity::new_string(str)
    } else {
      UnknownEntity::new_string()
    }
  }

  fn get_to_string(&self, _rc: &Entity<'a>) -> Entity<'a> {
    UnknownEntity::new_string()
  }

  fn get_to_numeric(&self, _rc: &Entity<'a>) -> Entity<'a> {
    UnknownEntity::new_unknown()
  }

  fn get_to_property_key(&self, _rc: &Entity<'a>) -> Entity<'a> {
    UnknownEntity::new_unknown()
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
      UnknownEntity::Object => TypeofResult::Object,
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

macro_rules! unknown_entity_ctors {
  ($($name:ident + $computed:ident -> $variant:ident,)*) => {
    $(
      pub fn $name() -> Entity<'a> {
        Entity::new(UnknownEntity::$variant)
      }

      pub fn $computed(dep: impl Into<Consumable<'a>>) -> Entity<'a> {
        ComputedEntity::new(Self::$name(), dep)
      }
    )*
  };
}

impl<'a> UnknownEntity {
  unknown_entity_ctors! {
    new_unknown  + new_computed_unknown -> Unknown,
    new_boolean  + new_computed_boolean -> Boolean,
    new_number   + new_computed_number -> Number,
    new_string   + new_computed_string -> String,
    new_bigint   + new_computed_bigint -> BigInt,
    new_symbol   + new_computed_symbol -> Symbol,
    new_function + new_computed_function -> Function,
    new_regexp   + new_computed_regexp -> Regexp,
    new_array    + new_computed_array -> Array,
    new_object   + new_computed_object -> Object,
  }

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

  fn get_prototype<'b>(&self, analyzer: &'b mut Analyzer<'a>) -> &'b Prototype<'a> {
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
