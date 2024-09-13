use super::{object::create_object_prototype, Prototype};

pub fn create_bigint_prototype<'a>() -> Prototype<'a> {
  create_object_prototype()
}
