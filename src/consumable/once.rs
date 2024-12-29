use super::ConsumableTrait;
use crate::analyzer::Analyzer;
use std::{cell::Cell, fmt::Debug, marker::PhantomData};

pub struct OnceConsumable<'a, T: ConsumableTrait<'a> + 'a>(Cell<Option<T>>, PhantomData<&'a ()>);

impl<'a, T: ConsumableTrait<'a> + 'a> Debug for OnceConsumable<'a, T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("OnceConsumable").finish()
  }
}

impl<'a, T: ConsumableTrait<'a> + 'a> ConsumableTrait<'a> for OnceConsumable<'a, T> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    if let Some(value) = self.0.take() {
      value.consume(analyzer)
    }
  }
}

impl<'a, T: ConsumableTrait<'a> + 'a> OnceConsumable<'a, T> {
  pub fn new(value: T) -> Self {
    Self(Cell::new(Some(value)), PhantomData)
  }
}
