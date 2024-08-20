use crate::{entity::Entity, TreeShaker};
use oxc::{ast::ast::Declaration, semantic::SymbolId};

#[derive(Debug, Default, Clone)]
pub struct Data {}

impl<'a> TreeShaker<'a> {
  pub(crate) fn exec_declaration(
    &mut self,
    node: &'a Declaration,
    need_symbol: Option<SymbolId>,
  ) -> Option<Entity> {
    let data = self.load_data::<Data>(node);

    self.current_declaration = Some(node);

    if need_symbol.is_some() {
      match node {
        Declaration::VariableDeclaration(node) => {
          let mut result: Option<Entity> = None;
          for declarator in &node.declarations {
            result = result.or(self.exec_variable_declarator(declarator, need_symbol));
          }
          result
        }
        Declaration::FunctionDeclaration(node) => {
          let s = node.id.as_ref().unwrap().symbol_id.get().unwrap();
          if need_symbol.is_some() {
            assert!(s == need_symbol.unwrap());
            Some(self.exec_function(node))
          } else {
            self.declare_symbol(s);
            // Function declaration has no side effect
            None
          }
        }
        _ => todo!(),
      }
    } else {
      None
    }
  }
}
