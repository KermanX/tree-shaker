use crate::{analyzer::Analyzer, ast::AstType2, transformer::Transformer};
use oxc::{
  ast::ast::FunctionBody,
  span::{GetSpan, Span},
};

const AST_TYPE: AstType2 = AstType2::FunctionBody;

#[derive(Debug, Default)]
struct Data {
  last_stmt: Option<Span>,
}

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_function_body(&mut self, node: &'a FunctionBody<'a>) {
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
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_function_body(&mut self, node: FunctionBody<'a>) -> FunctionBody<'a> {
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

    let transformed_statements = self.transform_statements(transformed_statements);
    self.ast_builder.function_body(span, directives, transformed_statements)
  }
}
