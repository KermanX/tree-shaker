use crate::{
  analyzer::Analyzer, ast::AstKind2, consumable::box_consumable, scope::CfScopeKind,
  transformer::Transformer,
};
use oxc::{
  ast::ast::{ForStatement, ForStatementInit, Statement},
  span::GetSpan,
};

impl<'a> Analyzer<'a> {
  pub fn exec_for_statement(&mut self, node: &'a ForStatement<'a>) {
    let labels = self.take_labels();

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

    let dep = if let Some(test) = &node.test {
      let test = self.exec_expression(test);
      if test.test_truthy() == Some(false) {
        return;
      }
      box_consumable((AstKind2::ForStatement(node), test))
    } else {
      box_consumable(AstKind2::ForStatement(node))
    };

    self.push_cf_scope_with_deps(
      CfScopeKind::BreakableWithoutLabel,
      labels.clone(),
      vec![dep],
      Some(false),
    );
    self.exec_loop(move |analyzer| {
      if analyzer.cf_scope().must_exited() {
        return;
      }

      analyzer.push_cf_scope(CfScopeKind::Continuable, labels.clone(), None);
      analyzer.exec_statement(&node.body);
      if let Some(update) = &node.update {
        analyzer.exec_expression(update);
      }
      analyzer.pop_cf_scope();

      if let Some(test) = &node.test {
        let test = analyzer.exec_expression(test);
        analyzer.cf_scope_mut().push_dep(box_consumable(test));
      }
    });
    self.pop_cf_scope();
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_for_statement(&self, node: &'a ForStatement<'a>) -> Option<Statement<'a>> {
    let ForStatement { span, init, test, update, body, .. } = node;

    if self.is_referred(AstKind2::ForStatement(node)) {
      let init = init
        .as_ref()
        .map(|init| match init {
          ForStatementInit::VariableDeclaration(node) => self
            .transform_variable_declaration(node)
            .map(|inner| ForStatementInit::VariableDeclaration(inner)),
          node => self
            .transform_expression(node.to_expression(), false)
            .map(|inner| ForStatementInit::from(inner)),
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
          ForStatementInit::VariableDeclaration(node) => self
            .transform_variable_declaration(node)
            .map(|inner| Statement::VariableDeclaration(inner)),
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
