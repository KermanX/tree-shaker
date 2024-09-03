use crate::{
  analyzer::Analyzer,
  ast::AstType2,
  entity::{dep::EntityDepNode, forwarded::ForwardedEntity},
  transformer::Transformer,
};
use oxc::{
  ast::ast::{ExpressionStatement, FunctionBody, Statement},
  span::{GetSpan, Span},
};

const AST_TYPE: AstType2 = AstType2::FunctionBody;

#[derive(Debug, Default)]
struct Data {
  last_stmt: Option<Span>,
}

impl<'a> Analyzer<'a> {
  pub fn exec_function_body(&mut self, node: &'a FunctionBody<'a>) {
    let mut span: Option<Span> = None;
    for statement in &node.statements {
      if self.cf_scope().must_exited() {
        break;
      }
      self.exec_statement(statement);
      span = Some(statement.span());
    }
    if let Some(span) = span {
      let data = self.load_data::<Data>(AST_TYPE, node);
      data.last_stmt = match data.last_stmt {
        Some(current_span) => Some(current_span.max(span)),
        None => Some(span),
      };
    }
  }

  pub fn exec_function_expression_body(&mut self, node: &'a FunctionBody<'a>) {
    debug_assert!(node.statements.len() == 1);
    if let Some(Statement::ExpressionStatement(expr)) = node.statements.first() {
      let dep = self.new_entity_dep(EntityDepNode::FunctionBodyAsExpression(node));
      let value = self.exec_expression(&expr.expression);
      let function_scope = self.function_scope_mut();
      function_scope.returned_value.push(ForwardedEntity::new(value, dep));
    } else {
      unreachable!();
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_function_body(&mut self, node: FunctionBody<'a>) -> FunctionBody<'a> {
    let data = self.get_data::<Data>(AST_TYPE, &node);

    let FunctionBody { span, directives, statements, .. } = node;

    let mut transformed_statements = self.ast_builder.vec();

    for statement in statements {
      let span = statement.span();

      if let Some(statement) = self.transform_statement(statement) {
        transformed_statements.push(statement);
      }

      if data.last_stmt == Some(span) {
        break;
      }
    }

    self.ast_builder.function_body(span, directives, transformed_statements)
  }

  pub fn transform_function_expression_body(&mut self, node: FunctionBody<'a>) -> FunctionBody<'a> {
    let need_val = self.is_referred(EntityDepNode::FunctionBodyAsExpression(&node));

    let FunctionBody { span, directives, statements, .. } = node;

    if let Some(Statement::ExpressionStatement(expr)) = statements.into_iter().next() {
      let ExpressionStatement { expression, .. } = expr.unbox();

      let expr = self.transform_expression(expression, need_val);

      self.ast_builder.function_body(
        span,
        directives,
        expr.map_or_else(
          || self.ast_builder.vec(),
          |expr| self.ast_builder.vec1(self.ast_builder.statement_expression(span, expr)),
        ),
      )
    } else {
      unreachable!();
    }
  }
}
