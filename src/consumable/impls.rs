use super::{Consumable, ConsumableTrait};
use crate::{analyzer::Analyzer, ast::AstKind2, dep::DepId, entity::Entity};
use std::{
  cell::{Cell, RefCell},
  rc::Rc,
};

impl<'a> ConsumableTrait<'a> for () {
  fn consume(&self, _: &mut Analyzer<'a>) {}
}

impl<'a, T: ConsumableTrait<'a> + 'a> ConsumableTrait<'a> for &'a T {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    (*self).consume(analyzer)
  }
}

impl<'a, T: ConsumableTrait<'a> + 'a> ConsumableTrait<'a> for Box<T> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    self.as_ref().consume(analyzer)
  }
}

impl<'a, T: ConsumableTrait<'a> + 'a> ConsumableTrait<'a> for Option<T> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    if let Some(value) = self {
      value.consume(analyzer)
    }
  }
}

impl<'a, T: Default + Copy + ConsumableTrait<'a> + 'a> ConsumableTrait<'a> for Cell<T> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    self.take().consume(analyzer)
  }
}

impl<'a, T: Default + ConsumableTrait<'a> + 'a> ConsumableTrait<'a> for RefCell<T> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    self.take().consume(analyzer)
  }
}

impl<'a, T: Default + ConsumableTrait<'a> + 'a> ConsumableTrait<'a> for Rc<T> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    self.as_ref().consume(analyzer)
  }
}

impl<'a> ConsumableTrait<'a> for Consumable<'a> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    self.0.consume(analyzer)
  }
}

impl<'a, T: ConsumableTrait<'a> + 'a> ConsumableTrait<'a> for Vec<T> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    for item in self {
      item.consume(analyzer)
    }
  }
}

impl<'a, T1: ConsumableTrait<'a> + 'a, T2: ConsumableTrait<'a> + 'a> ConsumableTrait<'a>
  for (T1, T2)
{
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    self.0.consume(analyzer);
    self.1.consume(analyzer)
  }
}

impl<
    'a,
    T1: ConsumableTrait<'a> + 'a,
    T2: ConsumableTrait<'a> + 'a,
    T3: ConsumableTrait<'a> + 'a,
  > ConsumableTrait<'a> for (T1, T2, T3)
{
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    self.0.consume(analyzer);
    self.1.consume(analyzer);
    self.2.consume(analyzer);
  }
}

impl<
    'a,
    T1: ConsumableTrait<'a> + 'a,
    T2: ConsumableTrait<'a> + 'a,
    T3: ConsumableTrait<'a> + 'a,
    T4: ConsumableTrait<'a> + 'a,
  > ConsumableTrait<'a> for (T1, T2, T3, T4)
{
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    self.0.consume(analyzer);
    self.1.consume(analyzer);
    self.2.consume(analyzer);
    self.3.consume(analyzer);
  }
}

impl<'a> ConsumableTrait<'a> for Entity<'a> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    self.consume(analyzer)
  }
}

impl<'a> ConsumableTrait<'a> for DepId {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    analyzer.refer_dep(*self);
  }
}

impl<'a> ConsumableTrait<'a> for AstKind2<'a> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    analyzer.refer_dep(*self);
  }
}
