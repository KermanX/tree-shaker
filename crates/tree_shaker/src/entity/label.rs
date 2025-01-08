use crate::{ast::AstKind2, dep::DepId};
use oxc::ast::ast::LabelIdentifier;

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
    AstKind2::LabelIdentifier(self.node).into()
  }
}
