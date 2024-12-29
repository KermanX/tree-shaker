use crate::{
  consumable::{Consumable, LazyConsumable},
  TreeShakeConfig,
};

use super::{
  arguments::ArgumentsEntity, Entity, LiteralEntity, PrimitiveEntity, PureBuiltinFnEntity,
  UnknownEntity,
};
use oxc::allocator::Allocator;
use std::cell::{Cell, RefCell};

pub struct EntityFactory<'a> {
  pub allocator: &'a Allocator,
  instance_id_counter: Cell<usize>,

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

  pub empty_arguments: Entity<'a>,
  pub unmatched_prototype_property: Entity<'a>,

  pub empty_consumable: Consumable<'a>,
  pub consumed_lazy_consumable: LazyConsumable<'a>,
}

impl<'a> EntityFactory<'a> {
  pub fn new(allocator: &'a Allocator, config: &TreeShakeConfig) -> EntityFactory<'a> {
    let r#true = Entity::new_in(LiteralEntity::Boolean(true), allocator);
    let r#false = Entity::new_in(LiteralEntity::Boolean(false), allocator);
    let nan = Entity::new_in(LiteralEntity::NaN, allocator);
    let null = Entity::new_in(LiteralEntity::Null, allocator);
    let undefined = Entity::new_in(LiteralEntity::Undefined, allocator);

    let immutable_unknown = Entity::new_in(UnknownEntity::new(), allocator);
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

    let empty_arguments = Entity::new_in(ArgumentsEntity::default(), allocator);
    let unmatched_prototype_property =
      if config.unmatched_prototype_property_as_undefined { undefined } else { immutable_unknown };

    let empty_consumable = Consumable(allocator.alloc(()));
    let consumed_lazy_consumable = LazyConsumable(allocator.alloc(RefCell::new(None)));

    EntityFactory {
      allocator,
      instance_id_counter: Cell::new(0),

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

      empty_arguments,
      unmatched_prototype_property,

      empty_consumable,
      consumed_lazy_consumable,
    }
  }

  pub fn alloc<T>(&self, val: T) -> &'a mut T {
    self.allocator.alloc(val)
  }

  pub fn alloc_instance_id(&self) -> usize {
    let id = self.instance_id_counter.get();
    self.instance_id_counter.set(id + 1);
    id
  }
}
