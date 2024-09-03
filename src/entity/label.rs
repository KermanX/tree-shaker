use super::dep::EntityDepNode;
use oxc::ast::ast::LabelIdentifier;

#[derive(Debug)]
pub struct LabelEntity<'a> {
  pub name: &'a str,
  pub node: EntityDepNode<'a>,
}

impl<'a> LabelEntity<'a> {
  pub fn new(node: &'a LabelIdentifier<'a>) -> Self {
    LabelEntity { name: &node.name, node: EntityDepNode::LabelIdentifier(&node) }
  }
}
