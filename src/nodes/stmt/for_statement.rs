use crate::{analyzer::Analyzer, ast::AstType2, scope::CfScopeKind, transformer::Transformer};
use oxc::{
  ast::ast::{ForStatement, ForStatementInit, Statement},
  span::GetSpan,
};

const AST_TYPE: AstType2 = AstType2::ForStatement;

#[derive(Debug, Default)]
pub struct Data {
  need_loop: bool,
}

impl<'a> Analyzer<'a> {
  pub fn exec_for_statement(&mut self, node: &'a ForStatement<'a>) {
    let labels = self.take_labels();

    let data = self.load_data::<Data>(AST_TYPE, node);

    if let Some(init) = &node.init {
      match init {
        ForStatementInit::VariableDeclaration(node) => {
          self.declare_variable_declaration(node, false);
          self.init_variable_declaration(node, None);
        }
        node => {
          self.exec_expression(node.to_expression());
        }
      }
    }

    if let Some(test) = &node.test {
      let test = self.exec_expression(test);
      if test.test_truthy() == Some(false) {
        return;
      }
      test.consume(self);
    }

    data.need_loop = true;

    self.push_cf_scope(CfScopeKind::BreakableWithoutLabel, labels.clone(), Some(false));
    self.exec_loop(move |analyzer| {
      analyzer.push_cf_scope(CfScopeKind::Continuable, labels.clone(), None);

      analyzer.exec_statement(&node.body);
      if let Some(update) = &node.update {
        analyzer.exec_expression(update);
      }
      if let Some(test) = &node.test {
        analyzer.exec_expression(test).consume(analyzer);
      }

      analyzer.pop_cf_scope();
    });
    self.pop_cf_scope();
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_for_statement(&self, node: &'a ForStatement<'a>) -> Option<Statement<'a>> {
    let data = self.get_data::<Data>(AST_TYPE, node);

    let ForStatement { span, init, test, update, body, .. } = node;

    if data.need_loop {
      let init = init
        .as_ref()
        .map(|init| match init {
          ForStatementInit::VariableDeclaration(node) => self
            .transform_variable_declaration(node)
            .map(|inner| self.ast_builder.for_statement_init_from_variable_declaration(inner)),
          node => self
            .transform_expression(node.to_expression(), false)
            .map(|inner| self.ast_builder.for_statement_init_expression(inner)),
        })
        .flatten();

      let test = test.as_ref().map(|test| self.transform_expression(test, true).unwrap());

      let update = update.as_ref().map(|update| self.transform_expression(update, false)).flatten();

      let body = self
        .transform_statement(body)
        .unwrap_or_else(|| self.ast_builder.statement_empty(body.span()));

      Some(self.ast_builder.statement_for(*span, init, test, update, body))
    } else {
      let init = init
        .as_ref()
        .map(|init| match init {
          ForStatementInit::VariableDeclaration(node) => {
            self.transform_variable_declaration(node).map(|inner| {
              self
                .ast_builder
                .statement_declaration(self.ast_builder.declaration_from_variable(inner))
            })
          }
          node => self
            .transform_expression(node.to_expression(), false)
            .map(|inner| self.ast_builder.statement_expression(inner.span(), inner)),
        })
        .flatten();

      let test = test
        .as_ref()
        .map(|test| self.transform_expression(test, false))
        .flatten()
        .map(|test| self.ast_builder.statement_expression(test.span(), test));

      match (init, test) {
        (Some(init), test) => {
          let mut statements = self.ast_builder.vec_with_capacity(2);
          statements.push(init);
          if let Some(test) = test {
            statements.push(test);
          }
          Some(self.ast_builder.statement_block(*span, statements))
        }
        (None, Some(test)) => Some(test),
        (None, None) => None,
      }
    }
  }
}
