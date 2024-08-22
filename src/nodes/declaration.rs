use crate::ast_type::AstType2;
use crate::{analyzer::Analyzer, transformer::Transformer};
use oxc::ast::ast::{Declaration, VariableDeclaration};

const AST_TYPE: AstType2 = AstType2::Declaration;

#[derive(Debug, Default, Clone)]
pub struct Data {}

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_declaration(&mut self, node: &'a Declaration<'a>) -> bool {
    match node {
      Declaration::VariableDeclaration(node) => {
        let mut init_effect = false;
        for declarator in &node.declarations {
          init_effect |= self.exec_variable_declarator(declarator);
        }
        init_effect
      }
      Declaration::FunctionDeclaration(node) => self.exec_function(node).0,
      _ => todo!(),
    }
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_declaration(&self, node: Declaration<'a>) -> Option<Declaration<'a>> {
    match node {
      Declaration::VariableDeclaration(node) => {
        let VariableDeclaration { span, kind, declarations, .. } = node.unbox();
        let mut transformed_decls = self.ast_builder.vec();
        for declarator in declarations {
          let declarator = self.transform_variable_declarator(declarator);
          if let Some(declarator) = declarator {
            transformed_decls.push(declarator);
          }
        }
        if transformed_decls.is_empty() {
          None
        } else {
          Some(self.ast_builder.declaration_variable(span, kind, transformed_decls, false))
        }
      }
      Declaration::FunctionDeclaration(node) => {
        self.transform_function(node.unbox()).map(|f| self.ast_builder.declaration_from_function(f))
      }
      _ => todo!(),
    }
  }
}
