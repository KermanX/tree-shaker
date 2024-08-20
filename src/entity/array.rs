use std::{cell::LazyCell, rc::Rc};

use super::Entity;

#[derive(Debug, Default, Clone)]
pub struct ArrayEntity {
  elements: Vec<Rc<Entity>>,
  pub rest: Option<Rc<Entity>>,
}

impl ArrayEntity {
  pub fn from_tuple(elements: &[Entity]) -> Self {
    ArrayEntity { elements: elements.into_iter().map(|e| Rc::new(e.clone())).collect(), rest: None }
  }

  pub fn as_tuple(&self) -> Option<Vec<Entity>> {
    todo!()
  }

  pub fn get_property(&self, key: &Entity) -> Rc<Entity> {
    match key.to_property_key() {
      Entity::StringLiteral(key) => {
        // TODO: builtin properties
        Rc::new(Entity::Unknown)
      }
      Entity::NonEmptyString(true) => Rc::new(Entity::Union(self.elements.clone()).simplify()),
      Entity::NonEmptyString(false) | Entity::UnknownString => Rc::new(Entity::Unknown),
      Entity::Symbol(key) => {
        // TODO: builtin properties
        Rc::new(Entity::Unknown)
      }
      Entity::UnknownSymbol => Rc::new(Entity::Unknown),
      Entity::Union(keys) => Rc::new(Entity::Union(
        keys.iter().map(|key| self.get_property(key)).collect::<Vec<Rc<Entity>>>(),
      )),
      _ => unreachable!(),
    }
  }
}

pub const UNKNOWN_ARRAY: LazyCell<ArrayEntity> = LazyCell::new(|| ArrayEntity::default());
