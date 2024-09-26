use super::{object::create_object_prototype, Prototype};
use crate::entity::PureBuiltinFnEntity;

pub fn create_array_prototype<'a>() -> Prototype<'a> {
  let mut prototype = create_object_prototype();

  prototype.insert("concat", PureBuiltinFnEntity::returns_array());
  prototype.insert("copyWithin", PureBuiltinFnEntity::returns_unknown() /* mutates_self */);
  prototype.insert("entries", PureBuiltinFnEntity::returns_array());
  prototype.insert("every", PureBuiltinFnEntity::returns_boolean());
  prototype.insert("fill", PureBuiltinFnEntity::returns_unknown() /* mutates_self */);
  prototype.insert("filter", PureBuiltinFnEntity::returns_array());
  prototype.insert("find", PureBuiltinFnEntity::returns_unknown());
  prototype.insert("findIndex", PureBuiltinFnEntity::returns_number());
  prototype.insert("findLast", PureBuiltinFnEntity::returns_unknown());
  prototype.insert("findLastIndex", PureBuiltinFnEntity::returns_number());
  prototype.insert("flat", PureBuiltinFnEntity::returns_array());
  prototype.insert("flatMap", PureBuiltinFnEntity::returns_array());
  prototype.insert("forEach", PureBuiltinFnEntity::returns_unknown());
  prototype.insert("includes", PureBuiltinFnEntity::returns_boolean());
  prototype.insert("indexOf", PureBuiltinFnEntity::returns_number());
  prototype.insert("join", PureBuiltinFnEntity::returns_string());
  prototype.insert("keys", PureBuiltinFnEntity::returns_unknown());
  prototype.insert("lastIndexOf", PureBuiltinFnEntity::returns_number());
  prototype.insert("map", PureBuiltinFnEntity::returns_array());
  prototype.insert("pop", PureBuiltinFnEntity::returns_unknown() /* mutates_self */);
  prototype.insert("push", PureBuiltinFnEntity::returns_number() /* mutates_self */);
  prototype.insert("reduce", PureBuiltinFnEntity::returns_unknown());
  prototype.insert("reduceRight", PureBuiltinFnEntity::returns_unknown());
  prototype.insert("reverse", PureBuiltinFnEntity::returns_unknown() /* mutates_self */);
  prototype.insert("shift", PureBuiltinFnEntity::returns_unknown() /* mutates_self */);
  prototype.insert("slice", PureBuiltinFnEntity::returns_array());
  prototype.insert("some", PureBuiltinFnEntity::returns_boolean());
  prototype.insert("sort", PureBuiltinFnEntity::returns_unknown() /* mutates_self */);
  prototype.insert("splice", PureBuiltinFnEntity::returns_array() /* mutates_self */);
  prototype.insert("unshift", PureBuiltinFnEntity::returns_number() /* mutates_self */);
  prototype.insert("values", PureBuiltinFnEntity::returns_unknown());

  prototype
}
