use super::{Consumable, ConsumableNode, ConsumableTrait};
use crate::analyzer::Analyzer;
use std::mem;

#[derive(Debug)]
pub struct ConsumableCollector<'a, T: ConsumableTrait<'a> + 'a = Consumable<'a>> {
  pub current: Vec<T>,
  pub node: Option<ConsumableNode<'a>>,
}

impl<'a, T: ConsumableTrait<'a> + 'a> Default for ConsumableCollector<'a, T> {
  fn default() -> Self {
    Self { current: Vec::new(), node: None }
  }
}

impl<'a, T: ConsumableTrait<'a> + 'a> ConsumableCollector<'a, T> {
  pub fn new(current: Vec<T>) -> Self {
    Self { current, node: None }
  }

  pub fn is_empty(&self) -> bool {
    self.current.is_empty() && self.node.is_none()
  }

  pub fn push(&mut self, value: T) {
    self.current.push(value);
  }

  pub fn try_collect(&mut self) -> Option<ConsumableNode<'a>> {
    if self.current.is_empty() {
      self.node.clone()
    } else {
      let current = mem::take(&mut self.current);
      let node = Some(if let Some(node) = self.node.take() {
        ConsumableNode::new_box((current, node))
      } else {
        ConsumableNode::new_box(current)
      });
      self.node = node.clone();
      node
    }
  }

  pub fn collect(&mut self) -> ConsumableNode<'a> {
    self.try_collect().unwrap_or_else(|| ConsumableNode::new_box(()))
  }

  pub fn consume_all(self, analyzer: &mut Analyzer<'a>) {
    for value in self.current {
      value.consume(analyzer);
    }

    if let Some(node) = self.node {
      node.consume(analyzer);
    }
  }

  pub fn force_clear(&mut self) {
    self.current.clear();
    self.node = None;
  }

  pub fn may_not_referred(&self) -> bool {
    if !self.current.is_empty() {
      return true;
    }
    if let Some(node) = &self.node {
      node.may_not_referred()
    } else {
      false
    }
  }
}
