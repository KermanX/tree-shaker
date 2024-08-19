use std::rc::Rc;

use crate::{entity::Entity, TreeShakerImpl};
use oxc::ast::ast::{LogicalExpression, LogicalOperator};

#[derive(Debug, Default, Clone)]
pub struct Data {
  left: bool,
  right: bool,
}

impl<'a> TreeShakerImpl<'a> {
  pub(crate) fn exec_logical_expression(&mut self, node: &'a LogicalExpression) -> Entity {
    let data = self.load_data::<Data>(node);

    let left_val = self.exec_expression(&node.left);
    let right_val = self.exec_expression(&node.right);

    match &node.operator {
      LogicalOperator::And => match left_val.to_boolean() {
        Entity::BooleanLiteral(true) => right_val,
        Entity::BooleanLiteral(false) => left_val,
        Entity::Union(_) => Entity::Union(vec![Rc::new(left_val), Rc::new(right_val)]),
        _ => unreachable!(),
      },
      _ => todo!(),
    }
  }
}
