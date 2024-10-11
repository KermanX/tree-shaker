use super::{object::create_object_prototype, Prototype};
use crate::entity::EntityFactory;

pub fn create_array_prototype<'a>(factory: &EntityFactory<'a>) -> Prototype<'a> {
  let mut prototype = create_object_prototype(factory);

  prototype.insert("concat", factory.pure_fn_returns_array);
  prototype.insert("copyWithin", factory.pure_fn_returns_unknown /* mutates_self */);
  prototype.insert("entries", factory.pure_fn_returns_array);
  prototype.insert("every", factory.pure_fn_returns_boolean);
  prototype.insert("fill", factory.pure_fn_returns_unknown /* mutates_self */);
  prototype.insert("filter", factory.pure_fn_returns_array);
  prototype.insert("find", factory.pure_fn_returns_unknown);
  prototype.insert("findIndex", factory.pure_fn_returns_number);
  prototype.insert("findLast", factory.pure_fn_returns_unknown);
  prototype.insert("findLastIndex", factory.pure_fn_returns_number);
  prototype.insert("flat", factory.pure_fn_returns_array);
  prototype.insert("flatMap", factory.pure_fn_returns_array);
  prototype.insert("forEach", factory.pure_fn_returns_unknown);
  prototype.insert("includes", factory.pure_fn_returns_boolean);
  prototype.insert("indexOf", factory.pure_fn_returns_number);
  prototype.insert("join", factory.pure_fn_returns_string);
  prototype.insert("keys", factory.pure_fn_returns_unknown);
  prototype.insert("lastIndexOf", factory.pure_fn_returns_number);
  prototype.insert("map", factory.pure_fn_returns_array);
  prototype.insert("pop", factory.pure_fn_returns_unknown /* mutates_self */);
  prototype.insert("push", factory.pure_fn_returns_number /* mutates_self */);
  prototype.insert("reduce", factory.pure_fn_returns_unknown);
  prototype.insert("reduceRight", factory.pure_fn_returns_unknown);
  prototype.insert("reverse", factory.pure_fn_returns_unknown /* mutates_self */);
  prototype.insert("shift", factory.pure_fn_returns_unknown /* mutates_self */);
  prototype.insert("slice", factory.pure_fn_returns_array);
  prototype.insert("some", factory.pure_fn_returns_boolean);
  prototype.insert("sort", factory.pure_fn_returns_unknown /* mutates_self */);
  prototype.insert("splice", factory.pure_fn_returns_array /* mutates_self */);
  prototype.insert("unshift", factory.pure_fn_returns_number /* mutates_self */);
  prototype.insert("values", factory.pure_fn_returns_unknown);

  prototype
}
