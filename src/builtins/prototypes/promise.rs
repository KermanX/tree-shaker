use super::{object::create_object_prototype, Prototype};
use crate::{entity::EntityFactory, init_prototype};

pub fn create_promise_prototype<'a>(factory: &EntityFactory<'a>) -> Prototype<'a> {
  init_prototype!("Promise", create_object_prototype(factory), {
    "finally" => factory.immutable_unknown,
    "then" => factory.immutable_unknown,
    "catch" => factory.immutable_unknown,
  })
}
