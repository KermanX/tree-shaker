use crate::{
  analyzer::Analyzer,
  entity::{Entity, LiteralEntity},
  host::Host,
};
use oxc::{
  ast::ast::{Expression, UnaryExpression, UnaryOperator},
  span::SPAN,
};
use oxc_ecmascript::ToInt32;

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_unary_expression(&mut self, node: &'a UnaryExpression) -> H::Entity {
    if node.operator == UnaryOperator::Delete {
      let dep = AstKind2::UnaryExpression(node);

      match &node.argument {
        Expression::StaticMemberExpression(node) => {
          let object = self.exec_expression(&node.object);
          let property = self.factory.string(&node.property.name);
          object.delete_property(self, self.consumable(dep), property)
        }
        Expression::PrivateFieldExpression(node) => {
          self.add_diagnostic("SyntaxError: private fields can't be deleted");
          let _object = self.exec_expression(&node.object);
          self.refer_dep(dep);
        }
        Expression::ComputedMemberExpression(node) => {
          let object = self.exec_expression(&node.object);
          let property = self.exec_expression(&node.expression);
          object.delete_property(self, self.consumable(dep), property)
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
        self.factory.computed(
          if let Some(num) = argument.get_literal(self).and_then(|lit| lit.to_number()) {
            if let Some(num) = num {
              let num = -num.0;
              self.factory.number(num, None)
            } else {
              self.factory.nan
            }
          } else {
            // Maybe number or bigint
            self.factory.unknown_primitive
          },
          argument,
        )
      }
      UnaryOperator::UnaryPlus => argument.get_to_numeric(self),
      UnaryOperator::LogicalNot => self.factory.computed(
        match argument.test_truthy() {
          Some(value) => self.factory.boolean(!value),
          None => self.factory.unknown_boolean,
        },
        argument,
      ),
      UnaryOperator::BitwiseNot => self.factory.computed(
        if let Some(literals) = argument.get_to_numeric(self).get_to_literals(self) {
          self.factory.union(
            literals
              .into_iter()
              .map(|lit| match lit {
                LiteralEntity::Number(num, _) => {
                  let num = !num.0.to_int_32();
                  self.factory.number(num as f64, None)
                }
                LiteralEntity::NaN => self.factory.number(-1f64, None),
                _ => self.factory.unknown_primitive,
              })
              .collect::<Vec<_>>(),
          )
        } else {
          self.factory.computed_unknown_primitive(argument)
        },
        argument,
      ),
      UnaryOperator::Typeof => argument.get_typeof(self),
      UnaryOperator::Void => self.factory.undefined,
      UnaryOperator::Delete => unreachable!(),
    }
  }
}
