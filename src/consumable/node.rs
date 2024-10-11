use super::{box_consumable, Consumable, ConsumableTrait};
use crate::analyzer::Analyzer;
use std::{cell::UnsafeCell, mem, rc::Rc};

#[derive(Debug, Clone)]
pub struct ConsumableNode<'a>(Rc<UnsafeCell<Option<Consumable<'a>>>>);

impl<'a> ConsumableNode<'a> {
  pub fn new(value: Consumable<'a>) -> Self {
    Self(Rc::new(UnsafeCell::new(Some(value))))
  }

  pub fn new_box(value: impl ConsumableTrait<'a> + 'a) -> Self {
    Self(Rc::new(UnsafeCell::new(Some(box_consumable(value)))))
  }
}

impl<'a> ConsumableTrait<'a> for ConsumableNode<'a> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    if let Some(value) = mem::replace(unsafe { &mut *self.0.get() }, None) {
      analyzer.consume(value);
    }
  }
  fn cloned(&self) -> Consumable<'a> {
    box_consumable(Clone::clone(self))
  }
}

impl<'a> Into<Consumable<'a>> for ConsumableNode<'a> {
  fn into(self) -> Consumable<'a> {
    box_consumable(self)
  }
}
