use crate::{analyzer::Analyzer, ast::AstType2, entity::entity::Entity, transformer::Transformer};
use oxc::ast::ast::{AwaitExpression, Expression};

const AST_TYPE: AstType2 = AstType2::AwaitExpression;

#[derive(Debug, Default)]
pub struct Data {
  has_effect: bool,
}

impl<'a> Analyzer<'a> {
  pub fn exec_await_expression(&mut self, node: &'a AwaitExpression<'a>) -> Entity<'a> {
    let value = self.exec_expression(&node.argument);
    let (has_effect, awaited) = value.r#await(self);

    let data = self.load_data::<Data>(AST_TYPE, node);
    data.has_effect |= has_effect;

    let call_scope = self.call_scope_mut();
    if !call_scope.is_async {
      // TODO: throw warning
    }
    call_scope.await_has_effect |= has_effect;

    awaited
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_await_expression(
    &self,
    node: &'a AwaitExpression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    let data = self.get_data::<Data>(AST_TYPE, node);
    let AwaitExpression { span, argument, .. } = node;

    if data.has_effect {
      let argument = self.transform_expression(argument, true).unwrap();
      Some(self.ast_builder.expression_await(*span, argument))
    } else {
      self.transform_expression(argument, need_val)
    }
  }
}
