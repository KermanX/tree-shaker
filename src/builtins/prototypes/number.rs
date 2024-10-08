use super::{object::create_object_prototype, Prototype};
use crate::entity::EntityFactory;

pub fn create_number_prototype<'a>(factory: &EntityFactory<'a>) -> Prototype<'a> {
  let mut prototype = create_object_prototype(factory);

  prototype.insert("toExponential", factory.pure_fn_returns_string);
  prototype.insert("toFixed", factory.pure_fn_returns_string);
  prototype.insert("toLocaleString", factory.pure_fn_returns_string);
  prototype.insert("toPrecision", factory.pure_fn_returns_string);
  prototype.insert("valueOf", factory.pure_fn_returns_number);

  prototype
}
