use std::{cell::LazyCell, rc::Rc};

use super::EntityValue;

#[derive(Debug, Default, Clone)]
pub struct ArrayEntity {
  elements: Vec<Rc<EntityValue>>,
  pub rest: Option<Rc<EntityValue>>,
}

impl ArrayEntity {
  pub fn from_tuple(elements: &[EntityValue]) -> Self {
    ArrayEntity { elements: elements.into_iter().map(|e| Rc::new(e.clone())).collect(), rest: None }
  }

  pub fn as_tuple(&self) -> Option<Vec<EntityValue>> {
    todo!()
  }

  pub fn get_property(&self, key: &EntityValue) -> Rc<EntityValue> {
    match key.to_property_key() {
      EntityValue::StringLiteral(key) => {
        // TODO: builtin properties
        Rc::new(EntityValue::Unknown)
      }
      EntityValue::NonEmptyString(true) => {
        Rc::new(EntityValue::Union(self.elements.clone()).simplify())
      }
      EntityValue::NonEmptyString(false) | EntityValue::UnknownString => {
        Rc::new(EntityValue::Unknown)
      }
      EntityValue::Symbol(key) => {
        // TODO: builtin properties
        Rc::new(EntityValue::Unknown)
      }
      EntityValue::UnknownSymbol => Rc::new(EntityValue::Unknown),
      EntityValue::Union(keys) => Rc::new(EntityValue::Union(
        keys.iter().map(|key| self.get_property(key)).collect::<Vec<Rc<EntityValue>>>(),
      )),
      _ => unreachable!(),
    }
  }
}

pub const UNKNOWN_ARRAY: LazyCell<ArrayEntity> = LazyCell::new(|| ArrayEntity::default());
