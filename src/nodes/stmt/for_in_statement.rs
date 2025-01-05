use crate::{
  analyzer::Analyzer, ast::AstKind2, entity::TypeofResult, scope::CfScopeKind,
  transformer::Transformer,
};
use oxc::{
  ast::ast::{ForInStatement, Statement},
  span::GetSpan,
};

impl<'a> Analyzer<'a> {
  pub fn exec_for_in_statement(&mut self, node: &'a ForInStatement<'a>) {
    let labels = self.take_labels();
    let right = self.exec_expression(&node.right);

    // FIXME: enumerate keys!
    right.consume(self);

    let types_have_no_keys: TypeofResult = TypeofResult::Undefined
      | TypeofResult::Boolean
      | TypeofResult::Number
      | TypeofResult::String
      | TypeofResult::Symbol;

    // TODO: empty object, simple function, array
    if (right.test_typeof() & !types_have_no_keys) == TypeofResult::_None
      || right.test_nullish() == Some(true)
    {
      return;
    }

    self.declare_for_statement_left(&node.left);

    self.push_cf_scope_with_deps(
      CfScopeKind::BreakableWithoutLabel,
      labels.clone(),
      vec![AstKind2::ForInStatement(node).into(), right.into()],
      Some(false),
    );
    self.exec_loop(move |analyzer| {
      analyzer.declare_for_statement_left(&node.left);
      analyzer.init_for_statement_left(&node.left, analyzer.factory.unknown_string);

      analyzer.push_cf_scope(CfScopeKind::Continuable, labels.clone(), None);
      analyzer.exec_statement(&node.body);
      analyzer.pop_cf_scope();
    });
    self.pop_cf_scope();
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_for_in_statement(&self, node: &'a ForInStatement<'a>) -> Option<Statement<'a>> {
    let ForInStatement { span, left, right, body, .. } = node;

    let need_loop = self.is_referred(AstKind2::ForInStatement(node));

    let left_span = left.span();
    let body_span = body.span();

    let left = self.transform_for_statement_left(left);
    let body = self.transform_statement(body);

    if !need_loop || (left.is_none() && body.is_none()) {
      return self
        .transform_expression(right, false)
        .map(|expr| self.ast_builder.statement_expression(*span, expr));
    }

    let right = self.transform_expression(right, true).unwrap();

    Some(self.ast_builder.statement_for_in(
      *span,
      left.unwrap_or_else(|| self.build_unused_for_statement_left(left_span)),
      right,
      body.unwrap_or_else(|| self.ast_builder.statement_empty(body_span)),
    ))
  }
}
