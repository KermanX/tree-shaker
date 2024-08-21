use crate::{analyzer::Analyzer, entity::Entity, transformer::Transformer};
use oxc::{ast::ast::Declaration, semantic::SymbolId};

#[derive(Debug, Default, Clone)]
pub struct Data {}

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_declaration(&mut self, node: &'a Declaration) -> Option<Entity> {
    todo!()
    // if need_symbol.is_some() {
    //   match node {
    //     Declaration::VariableDeclaration(node) => {
    //       let mut result: Option<Entity> = None;
    //       for declarator in &node.declarations {
    //         result = result.or(self.exec_variable_declarator(declarator, need_symbol));
    //       }
    //       result
    //     }
    //     Declaration::FunctionDeclaration(node) => {
    //       let s = node.id.as_ref().unwrap().symbol_id.get().unwrap();
    //       if need_symbol.is_some() {
    //         assert!(s == need_symbol.unwrap());
    //         Some(self.exec_function(node))
    //       } else {
    //         self.declare_symbol(s);
    //         // Function declaration has no side effect
    //         None
    //       }
    //     }
    //     _ => todo!(),
    //   }
    // } else {
    //   None
    // }
  }
}

impl<'a> Transformer<'a> {}
