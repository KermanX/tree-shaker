use super::Prototype;
use crate::entity::EntityFactory;

pub fn create_null_prototype<'a>(_factory: &EntityFactory<'a>) -> Prototype<'a> {
  Prototype::default().with_name("null")
}
