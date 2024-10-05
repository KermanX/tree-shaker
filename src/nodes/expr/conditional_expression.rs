use crate::{
  analyzer::Analyzer,
  ast::AstType2,
  build_effect,
  entity::{ComputedEntity, Entity, UnionEntity},
  scope::{conditional::ConditionalData, CfScopeKind},
  transformer::Transformer,
};
use oxc::ast::ast::{ConditionalExpression, Expression};

const AST_TYPE: AstType2 = AstType2::ConditionalExpression;

#[derive(Debug, Default)]
pub struct Data<'a> {
  maybe_true: bool,
  maybe_false: bool,
  conditional: ConditionalData<'a>,
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

    let historical_indeterminate = data.maybe_true && data.maybe_false;
    let current_indeterminate = maybe_true && maybe_false;

    self.push_conditional_cf_scope(
      &mut data.conditional,
      CfScopeKind::ConditionalExpression,
      test.clone(),
      historical_indeterminate,
      current_indeterminate,
    );
    let result = match (maybe_true, maybe_false) {
      (true, false) => self.exec_expression(&node.consequent),
      (false, true) => self.exec_expression(&node.alternate),
      (true, true) => {
        let consequent = self.exec_expression(&node.consequent);
        self.cf_scope_mut().exited = None;
        let alternate = self.exec_expression(&node.alternate);
        UnionEntity::new(vec![consequent, alternate])
      }
      _ => unreachable!(),
    };
    self.pop_cf_scope();

    ComputedEntity::new(result, test)
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_conditional_expression(
    &self,
    node: &'a ConditionalExpression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    let data = self.get_data::<Data>(AST_TYPE, node);

    let ConditionalExpression { span, test, consequent, alternate, .. } = node;

    let consequent =
      data.maybe_true.then(|| self.transform_expression(consequent, need_val)).flatten();
    let alternate =
      data.maybe_false.then(|| self.transform_expression(alternate, need_val)).flatten();

    let need_test_val = consequent.is_some() && alternate.is_some();
    let test = self.transform_expression(test, need_test_val);

    match (test, consequent, alternate) {
      (Some(test), Some(consequent), Some(alternate)) => {
        Some(self.ast_builder.expression_conditional(*span, test, consequent, alternate))
      }
      (test, Some(branch), None) | (test, None, Some(branch)) => {
        Some(build_effect!(self.ast_builder, *span, test; branch))
      }
      (Some(test), None, None) => Some(test),
      (None, None, None) => None,
      _ => unreachable!(),
    }
  }
}
