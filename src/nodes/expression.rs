use crate::{entity::Entity, TreeShakerImpl};
use oxc::ast::ast::Expression;

#[derive(Debug, Default, Clone)]
pub struct Data {}

impl<'a> TreeShakerImpl<'a> {
  pub(crate) fn exec_expression(&mut self, node: &'a Expression) -> Entity {
    let data = self.load_data::<Data>(node);

    match node {
      Expression::NumericLiteral(node) => self.exec_numeric_literal(node),
      Expression::StringLiteral(node) => self.exec_string_literal(node),
      Expression::BooleanLiteral(node) => self.exec_boolean_literal(node),
      Expression::Identifier(node) => self.exec_identifier_reference_read(node),
      Expression::LogicalExpression(node) => self.exec_logical_expression(node),

      _ => todo!(),
    }
  }
}
