use super::{Consumable, ConsumableNode, ConsumableTrait, ConsumableVec};
use crate::analyzer::Analyzer;
use std::mem;

#[derive(Debug, Default)]
pub struct ConsumableCollector<'a> {
  current: ConsumableVec<'a>,
  node: Option<ConsumableNode<'a>>,
}

impl<'a> ConsumableCollector<'a> {
  pub fn new(current: ConsumableVec<'a>) -> Self {
    Self { current, node: None }
  }

  pub fn push(&mut self, value: Consumable<'a>) {
    self.current.push(value);
  }

  pub fn try_collect(&mut self) -> Option<ConsumableNode<'a>> {
    let node = if self.current.is_empty() {
      if let Some(node) = self.node.take() {
        Some(ConsumableNode::new_box(node))
      } else {
        None
      }
    } else {
      let current = mem::take(&mut self.current);
      Some(if let Some(node) = self.node.take() {
        ConsumableNode::new(Box::new((current, node)))
      } else {
        ConsumableNode::new_box(current)
      })
    };
    self.node = node.clone();
    node
  }

  pub fn collect(&mut self) -> ConsumableNode<'a> {
    self.try_collect().unwrap_or_else(|| ConsumableNode::new_box(()))
  }

  pub fn consume_all(&mut self, analyzer: &mut Analyzer<'a>) {
    for value in mem::take(&mut self.current) {
      value.consume(analyzer);
    }

    if let Some(node) = mem::take(&mut self.node) {
      node.consume(analyzer);
    }
  }
}
