use crate::entity::EntityFactory;
use super::Prototype;

pub fn create_null_prototype<'a>(factory: &EntityFactory<'a>) -> Prototype<'a> {
  Prototype::new()
}
