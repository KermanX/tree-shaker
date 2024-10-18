use crate::dep::DepId;
use oxc::ast::{ast::LabelIdentifier, AstKind};

#[derive(Debug, Clone, Copy)]
pub struct LabelEntity<'a> {
  pub name: &'a str,
  pub node: &'a LabelIdentifier<'a>,
}

impl<'a> LabelEntity<'a> {
  pub fn new(node: &'a LabelIdentifier<'a>) -> Self {
    LabelEntity { name: &node.name, node }
  }

  pub fn dep_id(&self) -> DepId {
    AstKind::LabelIdentifier(self.node).into()
  }
}
