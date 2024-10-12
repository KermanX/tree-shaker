use crate::{
  analyzer::Analyzer, ast::AstType2, build_effect, entity::Entity, scope::CfScopeKind,
  transformer::Transformer,
};
use oxc::ast::{
  ast::{ConditionalExpression, Expression, LogicalOperator},
  AstKind,
};

const AST_TYPE: AstType2 = AstType2::ConditionalExpression;

#[derive(Debug, Default)]
pub struct Data {
  maybe_true: bool,
  maybe_false: bool,
}

impl<'a> Analyzer<'a> {
  pub fn exec_conditional_expression(&mut self, node: &'a ConditionalExpression<'a>) -> Entity<'a> {
    let test = self.exec_expression(&node.test);

    let (maybe_true, maybe_false) = match test.test_truthy() {
      Some(true) => (true, false),
      Some(false) => (false, true),
      None => (true, true),
    };

    let data = self.load_data::<Data>(AST_TYPE, node);
    data.maybe_true |= maybe_true;
    data.maybe_false |= maybe_false;

    let conditional_dep = self.push_conditional_cf_scope(
      AstKind::ConditionalExpression(node),
      CfScopeKind::ConditionalExpression,
      test.clone(),
      maybe_true,
      maybe_false,
    );
    let result = match (maybe_true, maybe_false) {
      (true, false) => self.exec_expression(&node.consequent),
      (false, true) => self.exec_expression(&node.alternate),
      (true, true) => {
        let consequent = self.exec_expression(&node.consequent);
        self.cf_scope_mut().exited = None;
        let alternate = self.exec_expression(&node.alternate);
        self.factory.union(vec![consequent, alternate])
      }
      _ => unreachable!(),
    };
    self.pop_cf_scope();

    self.factory.computed(result, conditional_dep)
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_conditional_expression(
    &self,
    node: &'a ConditionalExpression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    let data = self.get_data2::<Data>(AST_TYPE, node);

    let ConditionalExpression { span, test, consequent, alternate, .. } = node;

    let consequent =
      data.maybe_true.then(|| self.transform_expression(consequent, need_val)).flatten();
    let alternate =
      data.maybe_false.then(|| self.transform_expression(alternate, need_val)).flatten();

    let need_test_val = self.is_referred(AstKind::ConditionalExpression(node));
    let test = self.transform_expression(test, need_test_val);

    if need_test_val {
      let test = test.unwrap();

      match (consequent, alternate) {
        (Some(consequent), Some(alternate)) => {
          Some(self.ast_builder.expression_conditional(*span, test, consequent, alternate))
        }
        (Some(consequent), None) => {
          Some(self.ast_builder.expression_logical(*span, test, LogicalOperator::And, consequent))
        }
        (None, Some(alternate)) => {
          Some(self.ast_builder.expression_logical(*span, test, LogicalOperator::Or, alternate))
        }
        (None, None) => unreachable!(),
      }
    } else {
      build_effect!(self.ast_builder, *span, test, consequent, alternate)
    }
  }
}
