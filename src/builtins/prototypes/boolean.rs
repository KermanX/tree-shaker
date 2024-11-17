use super::{object::create_object_prototype, Prototype};
use crate::{entity::EntityFactory, init_prototype};

pub fn create_boolean_prototype<'a>(factory: &EntityFactory<'a>) -> Prototype<'a> {
  init_prototype!(create_object_prototype(factory), {
    "valueOf" => factory.pure_fn_returns_boolean,
  })
}
