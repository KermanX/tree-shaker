use super::{object::create_object_prototype, Prototype};
use crate::entity::builtin_fn::PureBuiltinFnEntity;

pub fn create_string_prototype<'a>() -> Prototype<'a> {
  let mut prototype = create_object_prototype();

  prototype.insert("anchor", PureBuiltinFnEntity::returns_string());
  prototype.insert("at", PureBuiltinFnEntity::returns_unknown());
  prototype.insert("big", PureBuiltinFnEntity::returns_string());
  prototype.insert("blink", PureBuiltinFnEntity::returns_string());
  prototype.insert("bold", PureBuiltinFnEntity::returns_string());
  prototype.insert("charAt", PureBuiltinFnEntity::returns_string());
  prototype.insert("charCodeAt", PureBuiltinFnEntity::returns_unknown());
  prototype.insert("codePointAt", PureBuiltinFnEntity::returns_unknown());
  prototype.insert("concat", PureBuiltinFnEntity::returns_string());
  prototype.insert("endsWith", PureBuiltinFnEntity::returns_unknown());
  prototype.insert("fixed", PureBuiltinFnEntity::returns_string());
  prototype.insert("fontcolor", PureBuiltinFnEntity::returns_string());
  prototype.insert("fontsize", PureBuiltinFnEntity::returns_string());
  prototype.insert("includes", PureBuiltinFnEntity::returns_unknown());
  prototype.insert("indexOf", PureBuiltinFnEntity::returns_unknown());
  prototype.insert("italics", PureBuiltinFnEntity::returns_string());
  prototype.insert("lastIndexOf", PureBuiltinFnEntity::returns_unknown());
  prototype.insert("link", PureBuiltinFnEntity::returns_string());
  prototype.insert("localeCompare", PureBuiltinFnEntity::returns_unknown());
  prototype.insert("match", PureBuiltinFnEntity::returns_unknown());
  prototype.insert("matchAll", PureBuiltinFnEntity::returns_unknown());
  prototype.insert("normalize", PureBuiltinFnEntity::returns_string());
  prototype.insert("padEnd", PureBuiltinFnEntity::returns_string());
  prototype.insert("padStart", PureBuiltinFnEntity::returns_string());
  prototype.insert("repeat", PureBuiltinFnEntity::returns_string());
  prototype.insert("replace", PureBuiltinFnEntity::returns_unknown());
  prototype.insert("replaceAll", PureBuiltinFnEntity::returns_unknown());
  prototype.insert("search", PureBuiltinFnEntity::returns_unknown());
  prototype.insert("slice", PureBuiltinFnEntity::returns_string());
  prototype.insert("small", PureBuiltinFnEntity::returns_string());
  prototype.insert("split", PureBuiltinFnEntity::returns_unknown());
  prototype.insert("startsWith", PureBuiltinFnEntity::returns_unknown());
  prototype.insert("strike", PureBuiltinFnEntity::returns_string());
  prototype.insert("sub", PureBuiltinFnEntity::returns_string());
  prototype.insert("substr", PureBuiltinFnEntity::returns_string());
  prototype.insert("substring", PureBuiltinFnEntity::returns_string());
  prototype.insert("sup", PureBuiltinFnEntity::returns_string());
  prototype.insert("toLocaleLowerCase", PureBuiltinFnEntity::returns_string());
  prototype.insert("toLocaleUpperCase", PureBuiltinFnEntity::returns_string());
  prototype.insert("toLowerCase", PureBuiltinFnEntity::returns_string());
  prototype.insert("toString", PureBuiltinFnEntity::returns_string());
  prototype.insert("toUpperCase", PureBuiltinFnEntity::returns_string());
  prototype.insert("trim", PureBuiltinFnEntity::returns_string());
  prototype.insert("trimEnd", PureBuiltinFnEntity::returns_string());
  prototype.insert("trimLeft", PureBuiltinFnEntity::returns_string());
  prototype.insert("trimRight", PureBuiltinFnEntity::returns_string());
  prototype.insert("trimStart", PureBuiltinFnEntity::returns_string());
  prototype.insert("valueOf", PureBuiltinFnEntity::returns_string());

  prototype
}
