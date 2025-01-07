use crate::entity::EntityFactory;

use super::{object::create_object_prototype, Prototype};

pub fn create_bigint_prototype<'a>(factory: &EntityFactory<'a>) -> Prototype<'a> {
  create_object_prototype(factory).with_name("BigInt")
}
