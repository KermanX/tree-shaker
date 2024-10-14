use crate::{
  analyzer::Analyzer, build_effect, consumable::box_consumable, entity::Entity,
  transformer::Transformer,
};
use oxc::{
  ast::{
    ast::{Expression, UnaryExpression, UnaryOperator},
    AstKind,
  },
  span::SPAN,
};

impl<'a> Analyzer<'a> {
  pub fn exec_unary_expression(&mut self, node: &'a UnaryExpression) -> Entity<'a> {
    if node.operator == UnaryOperator::Delete {
      let dep = AstKind::UnaryExpression(node);

      match &node.argument {
        Expression::StaticMemberExpression(node) => {
          let object = self.exec_expression(&node.object);
          let property = self.factory.string(&node.property.name);
          object.delete_property(self, box_consumable(dep), property)
        }
        Expression::PrivateFieldExpression(node) => {
          self.add_diagnostic("SyntaxError: private fields can't be deleted");
          let _object = self.exec_expression(&node.object);
          self.refer_dep(dep);
        }
        Expression::ComputedMemberExpression(node) => {
          let object = self.exec_expression(&node.object);
          let property = self.exec_expression(&node.expression);
          object.delete_property(self, box_consumable(dep), property)
        }
        Expression::Identifier(_node) => {
          self.add_diagnostic("SyntaxError: Delete of an unqualified identifier in strict mode");
          self.refer_dep(dep);
        }
        expr => {
          self.exec_expression(expr);
        }
      };

      return self.factory.r#true;
    }

    let argument = self.exec_expression(&node.argument);

    match &node.operator {
      UnaryOperator::UnaryNegation => {
        if let Some(num) = argument.get_literal(self).and_then(|lit| lit.to_number()) {
          if let Some(num) = num {
            let num = -num.0;
            self.factory.number(num, None)
          } else {
            self.factory.nan
          }
        } else {
          // Maybe number or bigint
          self.factory.computed_unknown(argument)
        }
      }
      UnaryOperator::UnaryPlus => argument.get_to_numeric(self),
      UnaryOperator::LogicalNot => match argument.test_truthy() {
        Some(true) => self.factory.r#false,
        Some(false) => self.factory.r#true,
        None => self.factory.computed_unknown_boolean(argument),
      },
      UnaryOperator::BitwiseNot => self.factory.computed_unknown(argument),
      UnaryOperator::Typeof => argument.get_typeof(self),
      UnaryOperator::Void => self.factory.undefined,
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
      return if self.is_referred(AstKind::UnaryExpression(node)) {
        let argument = match &node.argument {
          Expression::StaticMemberExpression(node) => {
            let object = self.transform_expression(&node.object, true).unwrap();
            self.ast_builder.expression_member(self.ast_builder.member_expression_static(
              node.span,
              object,
              node.property.clone(),
              node.optional,
            ))
          }
          Expression::PrivateFieldExpression(node) => {
            let object = self.transform_expression(&node.object, true).unwrap();
            self.ast_builder.expression_member(
              self.ast_builder.member_expression_private_field_expression(
                node.span,
                object,
                node.field.clone(),
                node.optional,
              ),
            )
          }
          Expression::ComputedMemberExpression(node) => {
            let object = self.transform_expression(&node.object, true).unwrap();
            let property = self.transform_expression(&node.expression, true).unwrap();
            self.ast_builder.expression_member(self.ast_builder.member_expression_computed(
              node.span,
              object,
              property,
              node.optional,
            ))
          }
          Expression::Identifier(node) => {
            self.ast_builder.expression_from_identifier_reference(self.clone_node(node))
          }
          _ => unreachable!(),
        };
        Some(self.ast_builder.expression_unary(*span, *operator, argument))
      } else {
        let expr = self.transform_expression(argument, false);
        if need_val {
          Some(build_effect!(
            &self.ast_builder,
            *span,
            self.transform_expression(argument, false);
            self.ast_builder.expression_boolean_literal(SPAN, true)
          ))
        } else {
          expr
        }
      };
    }

    let transformed_argument =
      self.transform_expression(argument, need_val && *operator != UnaryOperator::Void);

    match operator {
      UnaryOperator::UnaryNegation
      // FIXME: UnaryPlus can be removed if we have a number entity
      | UnaryOperator::UnaryPlus
      | UnaryOperator::LogicalNot
      | UnaryOperator::BitwiseNot
      | UnaryOperator::Typeof => {
        // `typeof unBoundIdentifier` does not throw a ReferenceError, but directly accessing `unBoundIdentifier` does. Thus we need to preserve the typeof operator.
        let should_preserve_typeof = transformed_argument.is_some() && *operator == UnaryOperator::Typeof && is_wrapped_identifier_reference(argument);

        if need_val || should_preserve_typeof {
          Some(self.ast_builder.expression_unary(*span, *operator, transformed_argument.unwrap()))
        } else {
          transformed_argument
        }
      }
      UnaryOperator::Void => match (need_val, transformed_argument) {
        (true, Some(argument)) => Some(self.ast_builder.expression_unary(*span, *operator, argument)),
        (true, None) => Some(self.build_undefined(*span)),
        (false, argument) => argument,
      },
      UnaryOperator::Delete => unreachable!(),
    }
  }
}

fn is_wrapped_identifier_reference<'a>(node: &'a Expression<'a>) -> bool {
  match node {
    Expression::Identifier(_) => true,
    Expression::ParenthesizedExpression(node) => is_wrapped_identifier_reference(&node.expression),
    _ => false,
  }
}
