use super::Entity;
use crate::{analyzer::Analyzer, consumable::Consumable};

pub fn boolean_from_test_result<'a>(
  analyzer: &Analyzer<'a>,
  result: Option<bool>,
  deps: impl FnOnce() -> Consumable<'a>,
) -> Entity<'a> {
  match result {
    Some(value) => analyzer.factory.new_boolean(value),
    None => analyzer.factory.new_computed(analyzer.factory.unknown_boolean, deps()),
  }
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
