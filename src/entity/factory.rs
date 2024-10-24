use super::{Entity, LiteralEntity, PrimitiveEntity, PureBuiltinFnEntity, UnknownEntity};
use oxc::allocator::Allocator;

pub struct EntityFactory<'a> {
  pub allocator: &'a Allocator,

  pub r#true: Entity<'a>,
  pub r#false: Entity<'a>,
  pub nan: Entity<'a>,
  pub null: Entity<'a>,
  pub undefined: Entity<'a>,

  pub immutable_unknown: Entity<'a>,

  pub unknown_primitive: Entity<'a>,
  pub unknown_string: Entity<'a>,
  pub unknown_number: Entity<'a>,
  pub unknown_bigint: Entity<'a>,
  pub unknown_boolean: Entity<'a>,
  pub unknown_symbol: Entity<'a>,

  pub pure_fn_returns_unknown: Entity<'a>,
  pub pure_fn_returns_string: Entity<'a>,
  pub pure_fn_returns_number: Entity<'a>,
  pub pure_fn_returns_bigint: Entity<'a>,
  pub pure_fn_returns_boolean: Entity<'a>,
  pub pure_fn_returns_symbol: Entity<'a>,
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

    let immutable_unknown = Entity::new_in(UnknownEntity::new_immutable(), allocator);
    let unknown_primitive = Entity::new_in(PrimitiveEntity::Mixed, allocator);
    let unknown_string = Entity::new_in(PrimitiveEntity::String, allocator);
    let unknown_number = Entity::new_in(PrimitiveEntity::Number, allocator);
    let unknown_bigint = Entity::new_in(PrimitiveEntity::BigInt, allocator);
    let unknown_boolean = Entity::new_in(PrimitiveEntity::Boolean, allocator);
    let unknown_symbol = Entity::new_in(PrimitiveEntity::Symbol, allocator);

    let pure_fn_returns_unknown =
      Entity::new_in(PureBuiltinFnEntity::new(|f| f.unknown()), allocator);

    let pure_fn_returns_string =
      Entity::new_in(PureBuiltinFnEntity::new(|f| f.unknown_string), allocator);
    let pure_fn_returns_number =
      Entity::new_in(PureBuiltinFnEntity::new(|f| f.unknown_number), allocator);
    let pure_fn_returns_bigint =
      Entity::new_in(PureBuiltinFnEntity::new(|f| f.unknown_bigint), allocator);
    let pure_fn_returns_boolean =
      Entity::new_in(PureBuiltinFnEntity::new(|f| f.unknown_boolean), allocator);
    let pure_fn_returns_symbol =
      Entity::new_in(PureBuiltinFnEntity::new(|f| f.unknown_symbol), allocator);
    let pure_fn_returns_null = Entity::new_in(PureBuiltinFnEntity::new(|f| f.null), allocator);
    let pure_fn_returns_undefined =
      Entity::new_in(PureBuiltinFnEntity::new(|f| f.undefined), allocator);

    EntityFactory {
      allocator,

      r#true,
      r#false,
      nan,
      null,
      undefined,

      immutable_unknown,

      unknown_primitive,
      unknown_string,
      unknown_number,
      unknown_bigint,
      unknown_boolean,
      unknown_symbol,

      pure_fn_returns_unknown,
      pure_fn_returns_string,
      pure_fn_returns_number,
      pure_fn_returns_bigint,
      pure_fn_returns_boolean,
      pure_fn_returns_symbol,
      pure_fn_returns_null,
      pure_fn_returns_undefined,
    }
  }

  pub fn alloc<T>(&self, val: T) -> &'a mut T {
    self.allocator.alloc(val)
  }
}
