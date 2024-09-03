use crate::{
  analyzer::Analyzer,
  ast::AstType2,
  entity::{
    typeof_result::TypeofResult,
    unknown::{UnknownEntity, UnknownEntityKind},
  },
  transformer::Transformer,
};
use oxc::{
  ast::ast::{ForInStatement, Statement},
  span::GetSpan,
};

const AST_TYPE: AstType2 = AstType2::ForInStatement;

#[derive(Debug, Default, Clone)]
pub struct Data {
  need_loop: bool,
}

impl<'a> Analyzer<'a> {
  pub fn exec_for_in_statement(&mut self, node: &'a ForInStatement<'a>) {
    let right = self.exec_expression(&node.right);

    // FIXME: enumerate keys!
    right.consume_as_unknown(self);

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

    let cf_scope_id = self.push_cf_scope(None, true);
    self.push_variable_scope(cf_scope_id);

    self.exec_for_statement_left(&node.left, UnknownEntity::new(UnknownEntityKind::Number));
    self.exec_statement(&node.body);

    self.pop_variable_scope();
    self.pop_cf_scope();

    let data = self.load_data::<Data>(AST_TYPE, node);
    data.need_loop = true;
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_for_in_statement(&mut self, node: ForInStatement<'a>) -> Option<Statement<'a>> {
    let data = self.get_data::<Data>(AST_TYPE, &node);

    let ForInStatement { span, left, right, body, .. } = node;

    let left_span = left.span();
    let body_span = body.span();

    let left = self.transform_for_statement_left(left);
    let body = self.transform_statement(body);

    if !data.need_loop || (left.is_none() && body.is_none()) {
      return self
        .transform_expression(right, false)
        .map(|expr| self.ast_builder.statement_expression(span, expr));
    }

    let right = self.transform_expression(right, true).unwrap();

    Some(self.ast_builder.statement_for_in(
      span,
      left.unwrap_or_else(|| self.build_unused_for_statement_left(left_span)),
      right,
      body.unwrap_or_else(|| self.ast_builder.statement_empty(body_span)),
    ))
  }
}
