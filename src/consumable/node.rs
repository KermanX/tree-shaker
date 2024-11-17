use super::{box_consumable, Consumable, ConsumableTrait};
use crate::analyzer::Analyzer;
use std::{cell::UnsafeCell, marker::PhantomData, mem, rc::Rc};

#[derive(Debug)]
pub struct ConsumableNode<'a, T: ConsumableTrait<'a> + 'a = Consumable<'a>>(
  Rc<UnsafeCell<Option<T>>>,
  PhantomData<&'a T>,
);

impl<'a, T: ConsumableTrait<'a> + 'a> ConsumableNode<'a, T> {
  pub fn new(value: T) -> Self {
    Self(Rc::new(UnsafeCell::new(Some(value))), PhantomData)
  }

  pub fn new_box(value: T) -> ConsumableNode<'a> {
    ConsumableNode::new(box_consumable(value))
  }

  pub fn may_not_referred(&self) -> bool {
    unsafe { &*self.0.get() }.is_some()
  }
}

impl<'a, T: ConsumableTrait<'a> + 'a> Clone for ConsumableNode<'a, T> {
  fn clone(&self) -> Self {
    Self(Rc::clone(&self.0), PhantomData)
  }
}

impl<'a, T: ConsumableTrait<'a> + 'a> ConsumableTrait<'a> for ConsumableNode<'a, T> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    if let Some(value) = mem::replace(unsafe { &mut *self.0.get() }, None) {
      analyzer.consume(value);
    }
  }
  fn cloned(&self) -> Consumable<'a> {
    box_consumable(self.clone())
  }
}
