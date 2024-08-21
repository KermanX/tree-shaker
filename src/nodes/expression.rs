use crate::{entity::Entity, TreeShaker};
use oxc::ast::ast::Expression;

#[derive(Debug, Default, Clone)]
pub struct Data {
  val: Entity
}

impl<'a> TreeShaker<'a> {
  pub(crate) fn exec_expression(&mut self, node: &'a Expression) -> Entity {
    let data = self.load_data::<Data>(node);

    data.val = match node {
      Expression::NumericLiteral(node) => self.exc_numeric_literal(node),
      Expression::StringLiteral(node) => self.exec_string_literal(node),
      Expression::BooleanLiteral(node) => self.exec_boolean_literal(node),
      Expression::Identifier(node) => self.exec_identifier_reference_read(node),
      Expression::LogicalExpression(node) => self.exec_logical_expression(node),
      Expression::CallExpression(node) => self.exec_call_expression(node),

      _ => todo!(),
    };

    data.val.clone()
  }

  pub(crate) fn calc_expression(&self, node: &'a Expression) -> Entity {
    let data = self.get_data::<Data>(node);

    data.val.clone()
  }

  pub(crate) fn transform_expression(
    &mut self,
    node: Expression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    match node {
      Expression::NumericLiteral(_)
      | Expression::StringLiteral(_)
      | Expression::BooleanLiteral(_)
      | Expression::Identifier(_) => {
        if need_val {
          Some(node)
        } else {
          None
        }
      }

      Expression::LogicalExpression(node) => {
        self.transform_logical_expression(node.unbox(), need_val)
      }

      Expression::CallExpression(node) => self.transform_call_expression(node.unbox(), need_val),

      _ => todo!(),
    }
  }
}
