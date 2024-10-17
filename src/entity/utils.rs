use super::Entity;
use crate::{analyzer::Analyzer, consumable::ConsumableTrait};

pub fn boolean_from_test_result<'a>(
  analyzer: &Analyzer<'a>,
  result: Option<bool>,
  dep: impl ConsumableTrait<'a> + 'a,
) -> Entity<'a> {
  analyzer.factory.computed(
    match result {
      Some(value) => analyzer.factory.boolean(value),
      None => analyzer.factory.unknown_boolean,
    },
    dep,
  )
}

#[macro_export]
macro_rules! use_consumed_flag {
  ($self: expr) => {
    if $self.consumed.get() {
      return;
    }
    $self.consumed.set(true);
  };
}
