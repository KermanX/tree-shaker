use super::{object::create_object_prototype, Prototype};
use crate::{entity::EntityFactory, init_prototype};

pub fn create_array_prototype<'a>(factory: &EntityFactory<'a>) -> Prototype<'a> {
  init_prototype!("Array", create_object_prototype(factory), {
    "concat" => factory.immutable_unknown /*pure_fn_returns_array*/,
    "copyWithin" => factory.pure_fn_returns_unknown /* mutates_self */,
    "entries" => factory.immutable_unknown /*pure_fn_returns_array*/,
    "every" => factory.pure_fn_returns_boolean,
    "fill" => factory.pure_fn_returns_unknown /* mutates_self */,
    "filter" => factory.immutable_unknown /*pure_fn_returns_array*/,
    "find" => factory.pure_fn_returns_unknown,
    "findIndex" => factory.pure_fn_returns_number,
    "findLast" => factory.pure_fn_returns_unknown,
    "findLastIndex" => factory.pure_fn_returns_number,
    "flat" => factory.immutable_unknown /*pure_fn_returns_array*/,
    "flatMap" => factory.immutable_unknown /*pure_fn_returns_array*/,
    "forEach" => factory.pure_fn_returns_unknown,
    "includes" => factory.pure_fn_returns_boolean,
    "indexOf" => factory.pure_fn_returns_number,
    "join" => factory.pure_fn_returns_string,
    "keys" => factory.pure_fn_returns_unknown,
    "lastIndexOf" => factory.pure_fn_returns_number,
    "map" => factory.immutable_unknown /*pure_fn_returns_array*/,
    "pop" => factory.pure_fn_returns_unknown /* mutates_self */,
    "push" => factory.pure_fn_returns_number /* mutates_self */,
    "reduce" => factory.pure_fn_returns_unknown,
    "reduceRight" => factory.pure_fn_returns_unknown,
    "reverse" => factory.pure_fn_returns_unknown /* mutates_self */,
    "shift" => factory.pure_fn_returns_unknown /* mutates_self */,
    "slice" => factory.immutable_unknown /*pure_fn_returns_array*/,
    "some" => factory.pure_fn_returns_boolean,
    "sort" => factory.pure_fn_returns_unknown /* mutates_self */,
    "splice" => factory.immutable_unknown /*pure_fn_returns_array*/ /* mutates_self */,
    "unshift" => factory.pure_fn_returns_number /* mutates_self */,
    "values" => factory.pure_fn_returns_unknown,
  })
}
