use crate::{entity::Entity, TreeShaker};
use oxc::{ast::ast::Declaration, semantic::SymbolId};

#[derive(Debug, Default, Clone)]
pub struct Data {
  included: bool,
}

impl<'a> TreeShaker<'a> {
  pub(crate) fn exec_declaration(
    &mut self,
    node: &'a Declaration,
    need_symbol: Option<SymbolId>,
  ) -> Option<Entity> {
    let data = self.load_data::<Data>(node);
    if !data.included {
      self.current_declaration = Some(node);
    }
    if !data.included || need_symbol.is_some() {
      let mut result: Option<Entity> = None;
      match node {
        Declaration::VariableDeclaration(node) => {
          for declarator in &node.declarations {
            result = result.or(self.exec_variable_declarator(declarator, need_symbol));
          }
        }
        Declaration::FunctionDeclaration(node) => {
          todo!();
        }
        _ => todo!(),
      };
      result
    } else {
      None
    }
  }
}
