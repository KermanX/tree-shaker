use crate::{entity::Entity, TreeShaker};
use oxc::{ast::ast::VariableDeclarator, semantic::SymbolId};
use rustc_hash::FxHashSet;

#[derive(Debug, Default, Clone)]
pub struct Data {
  included: FxHashSet<SymbolId>,
}

impl<'a> TreeShaker<'a> {
  pub(crate) fn exec_variable_declarator(
    &mut self,
    node: &'a VariableDeclarator,
    need_symbol: Option<SymbolId>,
  ) -> Option<Entity> {
    let data = self.load_data::<Data>(node);
    need_symbol.map(|symbol| data.included.insert(symbol));

    let init_val = match &node.init {
      Some(init) => self.exec_expression(init),
      None => Entity::Undefined,
    };

    self.exec_binding_pattern(&node.id, need_symbol, init_val)
  }
}
