use super::{Consumable, ConsumableTrait, ConsumableVec};
use crate::analyzer::Analyzer;
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Clone)]
pub struct LazyConsumable<'a>(Rc<RefCell<Option<ConsumableVec<'a>>>>);

impl<'a> ConsumableTrait<'a> for LazyConsumable<'a> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    self.0.consume(analyzer);
  }
}

impl<'a> LazyConsumable<'a> {
  pub fn new(consumable: Consumable<'a>) -> Self {
    Self(Rc::new(RefCell::new(Some(vec![consumable]))))
  }

  pub fn new_consumed() -> Self {
    Self(Rc::new(RefCell::new(None)))
  }

  pub fn push(&self, analyzer: &mut Analyzer<'a>, consumable: Consumable<'a>) {
    let mut comsumables_ref = self.0.borrow_mut();
    if let Some(consumables) = comsumables_ref.as_mut() {
      consumables.push(consumable);
    } else {
      drop(comsumables_ref);
      analyzer.consume(consumable);
    }
  }
}
