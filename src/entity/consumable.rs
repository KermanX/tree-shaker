use super::{Entity, EntityDepNode, EntityTrait};
use crate::{analyzer::Analyzer, transformer::Transformer};
use oxc::ast::AstKind;
use std::{fmt::Debug, ops::Deref, rc::Rc};

pub trait ConsumableInternal<'a>: Debug {
  fn consume(&self, analyzer: &mut Analyzer<'a>);

  /// FIXME: remove this
  fn refer_dep_shallow(&self, _transformer: &Transformer<'a>) {}
}

#[derive(Debug, Clone)]
pub struct Consumable<'a>(Rc<dyn ConsumableInternal<'a> + 'a>);

impl<'a> Consumable<'a> {
  pub fn new(consumable: impl ConsumableInternal<'a> + 'a) -> Self {
    Self(Rc::new(consumable))
  }
}

impl<'a, T: ConsumableInternal<'a> + 'a> From<T> for Consumable<'a> {
  fn from(consumable: T) -> Self {
    Self::new(consumable)
  }
}

impl<'a> From<AstKind<'a>> for Consumable<'a> {
  fn from(value: AstKind<'a>) -> Self {
    let dep_node: EntityDepNode = value.into();
    Self::new(dep_node)
  }
}

impl<'a> Deref for Consumable<'a> {
  type Target = Rc<dyn ConsumableInternal<'a> + 'a>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<'a> ConsumableInternal<'a> for () {
  fn consume(&self, _consumer: &mut Analyzer<'a>) {
    // Do nothing
  }
}

impl<'a> ConsumableInternal<'a> for EntityDepNode {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    analyzer.refer_dep(*self);
  }

  fn refer_dep_shallow(&self, transformer: &Transformer<'a>) {
    transformer.refer_dep(*self);
  }
}

impl<'a, T1: ConsumableTrait<'a>, T2: ConsumableTrait<'a>> ConsumableInternal<'a> for (T1, T2) {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    self.0.consume(analyzer);
    self.1.consume(analyzer);
  }
}

impl<'a, T: ConsumableTrait<'a>> ConsumableInternal<'a> for Vec<T> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    for item in self {
      item.consume(analyzer);
    }
  }
}

impl<'a, T: EntityTrait<'a>> ConsumableInternal<'a> for T {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    self.consume(analyzer);
  }
}

impl<'a> ConsumableInternal<'a> for Entity<'a> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    self.consume(analyzer);
  }
}

impl<'a> ConsumableInternal<'a> for Rc<dyn ConsumableInternal<'a> + 'a> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    self.as_ref().consume(analyzer);
  }
}

impl<'a> Analyzer<'a> {
  pub fn consume(&mut self, dep: impl ConsumableTrait<'a>) {
    dep.consume(self);
  }
}

pub trait ConsumableTrait<'a>: Debug {
  fn consume(&self, analyzer: &mut Analyzer<'a>);
}

impl<'a, T: ConsumableInternal<'a> + 'a> ConsumableTrait<'a> for T {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    self.consume(analyzer);
  }
}

impl<'a> ConsumableTrait<'a> for Consumable<'a> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    self.0.as_ref().consume(analyzer);
  }
}
