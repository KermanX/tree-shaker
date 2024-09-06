use crate::{
  analyzer::Analyzer,
  ast::AstType2,
  entity::{
    entity::Entity,
    literal::LiteralEntity,
    unknown::{UnknownEntity, UnknownEntityKind},
  },
  transformer::Transformer,
};
use oxc::ast::ast::{Expression, UnaryExpression, UnaryOperator};

const AST_TYPE: AstType2 = AstType2::UnaryExpression;

#[derive(Debug, Default)]
pub struct Data {
  need_delete: bool,
}

impl<'a> Analyzer<'a> {
  pub fn exec_unary_expression(&mut self, node: &'a UnaryExpression) -> Entity<'a> {
    if node.operator == UnaryOperator::Delete {
      let data = self.load_data::<Data>(AST_TYPE, node);

      data.need_delete |= match &node.argument {
        Expression::StaticMemberExpression(node) => {
          let object = self.exec_expression(&node.object);
          let property = LiteralEntity::new_string(&node.property.name);
          object.delete_property(self, &property)
        }
        Expression::PrivateFieldExpression(_node) => {
          // TODO: throw warning: SyntaxError: private fields can't be deleted
          true
        }
        Expression::ComputedMemberExpression(node) => {
          let object = self.exec_expression(&node.object);
          let property = self.exec_expression(&node.expression);
          object.delete_property(self, &property)
        }
        _ => false,
      };

      return LiteralEntity::new_boolean(true);
    }

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
      UnaryOperator::Delete => unreachable!(),
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_unary_expression(
    &self,
    node: &'a UnaryExpression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    let UnaryExpression { span, operator, argument } = node;

    if *operator == UnaryOperator::Delete {
      let data = self.get_data::<Data>(AST_TYPE, node);

      return if data.need_delete {
        let argument = self.transform_expression(argument, true).unwrap();
        Some(self.ast_builder.expression_unary(*span, *operator, argument))
      } else {
        self.transform_expression(argument, false)
      };
    }

    let argument =
      self.transform_expression(argument, need_val && *operator != UnaryOperator::Void);

    match operator {
      UnaryOperator::UnaryNegation
      // FIXME: UnaryPlus can be removed if we have a number entity
      | UnaryOperator::UnaryPlus
      | UnaryOperator::LogicalNot
      | UnaryOperator::BitwiseNot
      | UnaryOperator::Typeof => {
        if need_val {
          Some(self.ast_builder.expression_unary(*span, *operator, argument.unwrap()))
        } else {
          argument
        }
      }
      UnaryOperator::Void => match (need_val, argument) {
        (true, Some(argument)) => Some(self.ast_builder.expression_unary(*span, *operator, argument)),
        (true, None) => Some(self.build_undefined(*span)),
        (false, argument) => argument,
      },
      UnaryOperator::Delete => unreachable!(),
    }
  }
}
