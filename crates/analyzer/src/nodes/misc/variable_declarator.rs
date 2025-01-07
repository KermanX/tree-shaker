use crate::{host::Host, 
  analyzer::Analyzer,
  ast::{AstKind2, DeclarationKind},
  entity::Entity,
  };
use oxc::{ast::ast::VariableDeclarator, span::GetSpan};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn declare_variable_declarator(
    &mut self,
    node: &'a VariableDeclarator,
    exporting: bool,
    kind: DeclarationKind,
  ) {
    self.declare_binding_pattern(&node.id, exporting, kind);
  }

  pub fn init_variable_declarator(
    &mut self,
    node: &'a VariableDeclarator,
    init: Option<H::Entity>,
  ) {
    let init = match init {
      Some(init) => {
        if node.init.is_some() {
          self.thrown_builtin_error(
            "for-in/for-of loop variable declaration may not have an initializer",
          );
        }
        Some(init)
      }
      None => node.init.as_ref().map(|init| {
        let val = self.exec_expression(init);
        self.factory.computed(val, AstKind2::VariableDeclarator(node))
      }),
    };

    self.init_binding_pattern(&node.id, init);
  }
}

