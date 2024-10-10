use super::{object::create_object_prototype, Prototype};
use crate::entity::EntityFactory;

pub fn create_string_prototype<'a>(factory: &EntityFactory<'a>) -> Prototype<'a> {
  let mut prototype = create_object_prototype(factory);

  prototype.insert("anchor", factory.pure_fn_returns_string);
  prototype.insert("at", factory.pure_fn_returns_unknown);
  prototype.insert("big", factory.pure_fn_returns_string);
  prototype.insert("blink", factory.pure_fn_returns_string);
  prototype.insert("bold", factory.pure_fn_returns_string);
  prototype.insert("charAt", factory.pure_fn_returns_string);
  prototype.insert("charCodeAt", factory.pure_fn_returns_unknown);
  prototype.insert("codePointAt", factory.pure_fn_returns_unknown);
  prototype.insert("concat", factory.pure_fn_returns_string);
  prototype.insert("endsWith", factory.pure_fn_returns_unknown);
  prototype.insert("fixed", factory.pure_fn_returns_string);
  prototype.insert("fontcolor", factory.pure_fn_returns_string);
  prototype.insert("fontsize", factory.pure_fn_returns_string);
  prototype.insert("includes", factory.pure_fn_returns_unknown);
  prototype.insert("indexOf", factory.pure_fn_returns_unknown);
  prototype.insert("italics", factory.pure_fn_returns_string);
  prototype.insert("lastIndexOf", factory.pure_fn_returns_unknown);
  prototype.insert("link", factory.pure_fn_returns_string);
  prototype.insert("localeCompare", factory.pure_fn_returns_unknown);
  prototype.insert("match", factory.pure_fn_returns_unknown);
  prototype.insert("matchAll", factory.pure_fn_returns_unknown);
  prototype.insert("normalize", factory.pure_fn_returns_string);
  prototype.insert("padEnd", factory.pure_fn_returns_string);
  prototype.insert("padStart", factory.pure_fn_returns_string);
  prototype.insert("repeat", factory.pure_fn_returns_string);
  prototype.insert("replace", factory.pure_fn_returns_unknown);
  prototype.insert("replaceAll", factory.pure_fn_returns_unknown);
  prototype.insert("search", factory.pure_fn_returns_unknown);
  prototype.insert("slice", factory.pure_fn_returns_string);
  prototype.insert("small", factory.pure_fn_returns_string);
  prototype.insert("split", factory.pure_fn_returns_unknown);
  prototype.insert("startsWith", factory.pure_fn_returns_unknown);
  prototype.insert("strike", factory.pure_fn_returns_string);
  prototype.insert("sub", factory.pure_fn_returns_string);
  prototype.insert("substr", factory.pure_fn_returns_string);
  prototype.insert("substring", factory.pure_fn_returns_string);
  prototype.insert("sup", factory.pure_fn_returns_string);
  prototype.insert("toLocaleLowerCase", factory.pure_fn_returns_string);
  prototype.insert("toLocaleUpperCase", factory.pure_fn_returns_string);
  prototype.insert("toLowerCase", factory.pure_fn_returns_string);
  prototype.insert("toString", factory.pure_fn_returns_string);
  prototype.insert("toUpperCase", factory.pure_fn_returns_string);
  prototype.insert("trim", factory.pure_fn_returns_string);
  prototype.insert("trimEnd", factory.pure_fn_returns_string);
  prototype.insert("trimLeft", factory.pure_fn_returns_string);
  prototype.insert("trimRight", factory.pure_fn_returns_string);
  prototype.insert("trimStart", factory.pure_fn_returns_string);
  prototype.insert("valueOf", factory.pure_fn_returns_string);

  prototype
}
