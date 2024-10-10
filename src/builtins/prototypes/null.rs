use super::Prototype;
use crate::entity::EntityFactory;

pub fn create_null_prototype<'a>(factory: &EntityFactory<'a>) -> Prototype<'a> {
  Prototype::new()
}
