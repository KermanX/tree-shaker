use crate::ast_type::AstType2;
use crate::{entity::Entity, transformer::Transformer, Analyzer};
use oxc::ast::ast::Expression;

const AST_TYPE: AstType2 = AstType2::Expression;

#[derive(Debug, Default, Clone)]
pub struct Data {
  val: Entity,
}

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_expression(&mut self, node: &'a Expression) -> (bool, Entity) {
    let val = match node {
      Expression::NumericLiteral(node) => self.exc_numeric_literal(node),
      Expression::StringLiteral(node) => self.exec_string_literal(node),
      Expression::BooleanLiteral(node) => self.exec_boolean_literal(node),
      Expression::Identifier(node) => self.exec_identifier_reference_read(node),
      Expression::LogicalExpression(node) => self.exec_logical_expression(node),
      Expression::CallExpression(node) => self.exec_call_expression(node),
      Expression::ObjectExpression(node) => self.exec_object_expression(node),
      _ => todo!(),
    };

    self.set_data(AST_TYPE, node, Data { val: val.1.clone() });

    val
  }

  pub(crate) fn calc_expression(&self, node: &'a Expression) -> Entity {
    let data = self.get_data::<Data>(AST_TYPE, node);

    data.val.clone()
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_expression(
    &self,
    node: Expression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    match node {
      Expression::NumericLiteral(_)
      | Expression::StringLiteral(_)
      | Expression::BooleanLiteral(_) => {
        if need_val {
          Some(node)
        } else {
          None
        }
      }

      Expression::Identifier(node) => self
        .transform_identifier_reference_read(node.unbox(), need_val)
        .map(|id| self.ast_builder.expression_from_identifier_reference(id)),
      Expression::LogicalExpression(node) => {
        self.transform_logical_expression(node.unbox(), need_val)
      }

      Expression::CallExpression(node) => self.transform_call_expression(node.unbox(), need_val),

      Expression::ObjectExpression(node) => self.transform_object_expression(node.unbox()),

      _ => todo!(),
    }
  }
}
