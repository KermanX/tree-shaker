use super::{object::create_object_prototype, Prototype};
use crate::entity::UnknownEntity;

pub fn create_promise_prototype<'a>() -> Prototype<'a> {
  let mut prototype = create_object_prototype();

  prototype.insert("finally", UnknownEntity::new_unknown());
  prototype.insert("then", UnknownEntity::new_unknown());
  prototype.insert("catch", UnknownEntity::new_unknown());

  prototype
}
