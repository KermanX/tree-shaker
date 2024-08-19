use crate::{entity::Entity, TreeShakerImpl};
use oxc::{ast::ast::VariableDeclarator, semantic::SymbolId};
use rustc_hash::FxHashSet;

#[derive(Debug, Default, Clone)]
pub struct Data {
  included: FxHashSet<SymbolId>,
}

impl<'a> TreeShakerImpl<'a> {
  pub(crate) fn exec_variable_declarator(
    &mut self,
    node: &'a VariableDeclarator,
    need_symbol: Option<SymbolId>,
  ) -> Option<Entity> {
    let data = self.load_data::<Data>(node);
    need_symbol.map(|symbol| data.included.insert(symbol));

    let init_val = node
      .init
      .as_ref()
      .map_or(Entity::Undefined, |init| self.exec_expression(init, need_symbol.is_some()));

    self.exec_binding_pattern(&node.id, need_symbol, init_val)
  }
}
