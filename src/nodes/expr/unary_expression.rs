use crate::{
  analyzer::Analyzer,
  entity::{
    entity::Entity,
    literal::LiteralEntity,
    unknown::{UnknownEntity, UnknownEntityKind},
  },
  transformer::Transformer,
};
use oxc::ast::ast::{Expression, UnaryExpression, UnaryOperator};

impl<'a> Analyzer<'a> {
  pub fn exec_unary_expression(&mut self, node: &'a UnaryExpression) -> Entity<'a> {
    let argument = self.exec_expression(&node.argument);

    match &node.operator {
      UnaryOperator::UnaryNegation => {
        todo!()
      }
      UnaryOperator::UnaryPlus => {
        todo!()
      }
      UnaryOperator::LogicalNot => match argument.test_truthy() {
        Some(true) => LiteralEntity::new_boolean(false),
        Some(false) => LiteralEntity::new_boolean(true),
        None => UnknownEntity::new_with_deps(UnknownEntityKind::Boolean, vec![argument]),
      },
      UnaryOperator::BitwiseNot => {
        todo!()
      }
      UnaryOperator::Typeof => argument.get_typeof(),
      UnaryOperator::Void => LiteralEntity::new_undefined(),
      UnaryOperator::Delete => {
        todo!()
      }
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_unary_expression(
    &self,
    node: UnaryExpression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    let UnaryExpression { span, operator, argument } = node;

    let argument = self.transform_expression(argument, need_val && operator != UnaryOperator::Void);

    match operator {
      UnaryOperator::UnaryNegation
      // FIXME: UnaryPlus can be removed if we have a number entity
      | UnaryOperator::UnaryPlus
      | UnaryOperator::LogicalNot
      | UnaryOperator::BitwiseNot
      | UnaryOperator::Typeof => {
        if need_val {
          Some(self.ast_builder.expression_unary(span, operator, argument.unwrap()))
        } else {
          argument
        }
      }
      UnaryOperator::Void => match (need_val, argument) {
        (true, Some(argument)) => Some(self.ast_builder.expression_unary(span, operator, argument)),
        (true, None) => Some(self.build_undefined(span)),
        (false, argument) => argument,
      },
      UnaryOperator::Delete => {
        todo!()
      }
    }
  }
}
