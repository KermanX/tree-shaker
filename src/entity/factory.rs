use super::{Entity, LiteralEntity, PureBuiltinFnEntity, UnknownEntity};
use oxc::allocator::Allocator;

pub struct EntityFactory<'a> {
  pub allocator: &'a Allocator,

  pub r#true: Entity<'a>,
  pub r#false: Entity<'a>,
  pub nan: Entity<'a>,
  pub null: Entity<'a>,
  pub undefined: Entity<'a>,

  pub unknown_string: Entity<'a>,
  pub unknown_number: Entity<'a>,
  pub unknown_bigint: Entity<'a>,
  pub unknown_boolean: Entity<'a>,
  pub unknown_symbol: Entity<'a>,
  pub unknown_function: Entity<'a>,
  pub unknown_regexp: Entity<'a>,
  pub unknown_array: Entity<'a>,
  pub unknown_object: Entity<'a>,
  pub unknown: Entity<'a>,

  pub pure_fn_returns_unknown: Entity<'a>,
  pub pure_fn_returns_string: Entity<'a>,
  pub pure_fn_returns_number: Entity<'a>,
  pub pure_fn_returns_bigint: Entity<'a>,
  pub pure_fn_returns_boolean: Entity<'a>,
  pub pure_fn_returns_symbol: Entity<'a>,
  pub pure_fn_returns_function: Entity<'a>,
  pub pure_fn_returns_regexp: Entity<'a>,
  pub pure_fn_returns_array: Entity<'a>,
  pub pure_fn_returns_object: Entity<'a>,
  pub pure_fn_returns_null: Entity<'a>,
  pub pure_fn_returns_undefined: Entity<'a>,
}

impl<'a> EntityFactory<'a> {
  pub fn new(allocator: &Allocator) -> EntityFactory {
    let r#true = Entity::new_in(LiteralEntity::Boolean(true), allocator);
    let r#false = Entity::new_in(LiteralEntity::Boolean(false), allocator);
    let nan = Entity::new_in(LiteralEntity::NaN, allocator);
    let null = Entity::new_in(LiteralEntity::Null, allocator);
    let undefined = Entity::new_in(LiteralEntity::Undefined, allocator);

    let unknown_string = Entity::new_in(UnknownEntity::String, allocator);
    let unknown_number = Entity::new_in(UnknownEntity::Number, allocator);
    let unknown_bigint = Entity::new_in(UnknownEntity::BigInt, allocator);
    let unknown_boolean = Entity::new_in(UnknownEntity::Boolean, allocator);
    let unknown_symbol = Entity::new_in(UnknownEntity::Symbol, allocator);
    let unknown_function = Entity::new_in(UnknownEntity::Function, allocator);
    let unknown_regexp = Entity::new_in(UnknownEntity::Regexp, allocator);
    let unknown_array = Entity::new_in(UnknownEntity::Array, allocator);
    let unknown_object = Entity::new_in(UnknownEntity::Object, allocator);
    let unknown = Entity::new_in(UnknownEntity::Unknown, allocator);

    let pure_fn_returns_unknown = Entity::new_in(PureBuiltinFnEntity::new(unknown), allocator);
    let pure_fn_returns_string =
      Entity::new_in(PureBuiltinFnEntity::new(unknown_string), allocator);
    let pure_fn_returns_number =
      Entity::new_in(PureBuiltinFnEntity::new(unknown_number), allocator);
    let pure_fn_returns_bigint =
      Entity::new_in(PureBuiltinFnEntity::new(unknown_bigint), allocator);
    let pure_fn_returns_boolean =
      Entity::new_in(PureBuiltinFnEntity::new(unknown_boolean), allocator);
    let pure_fn_returns_symbol =
      Entity::new_in(PureBuiltinFnEntity::new(unknown_symbol), allocator);
    let pure_fn_returns_function =
      Entity::new_in(PureBuiltinFnEntity::new(unknown_function), allocator);
    let pure_fn_returns_regexp =
      Entity::new_in(PureBuiltinFnEntity::new(unknown_regexp), allocator);
    let pure_fn_returns_array = Entity::new_in(PureBuiltinFnEntity::new(unknown_array), allocator);
    let pure_fn_returns_object =
      Entity::new_in(PureBuiltinFnEntity::new(unknown_object), allocator);
    let pure_fn_returns_null = Entity::new_in(PureBuiltinFnEntity::new(null), allocator);
    let pure_fn_returns_undefined = Entity::new_in(PureBuiltinFnEntity::new(undefined), allocator);

    EntityFactory {
      allocator,

      r#true,
      r#false,
      nan,
      null,
      undefined,

      unknown_string,
      unknown_number,
      unknown_bigint,
      unknown_boolean,
      unknown_symbol,
      unknown_function,
      unknown_regexp,
      unknown_array,
      unknown_object,
      unknown,

      pure_fn_returns_unknown,
      pure_fn_returns_string,
      pure_fn_returns_number,
      pure_fn_returns_bigint,
      pure_fn_returns_boolean,
      pure_fn_returns_symbol,
      pure_fn_returns_function,
      pure_fn_returns_regexp,
      pure_fn_returns_array,
      pure_fn_returns_object,
      pure_fn_returns_null,
      pure_fn_returns_undefined,
    }
  }

  pub fn alloc<T>(&self, val: T) -> &'a mut T {
    self.allocator.alloc(val)
  }
}
