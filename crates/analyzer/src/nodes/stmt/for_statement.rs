use crate::{host::Host, analyzer::Analyzer,  scoping::CfScopeKind};
use oxc::{
  ast::ast::{ForStatement, ForStatementInit, Statement},
  span::GetSpan,
};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
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
      self.consumable((AstKind2::ForStatement(node), test))
    } else {
      self.consumable(AstKind2::ForStatement(node))
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
      analyzer.init_statement(&node.body);
      if let Some(update) = &node.update {
        analyzer.exec_expression(update);
      }
      analyzer.pop_cf_scope();

      if let Some(test) = &node.test {
        let test = analyzer.exec_expression(test);
        let test = analyzer.consumable(test);
        analyzer.cf_scope_mut().push_dep(test);
      }
    });
    self.pop_cf_scope();
  }
}

