use crate::{entity::Entity, symbol::SymbolSource, TreeShaker};
use oxc::{allocator::Vec, ast::ast::FormalParameter, semantic::SymbolId};

use super::binding_pattern::BindingPatternSource;

#[derive(Debug, Default, Clone)]
pub struct Data<'a> {
  arg: SymbolSource<'a>,
}

impl<'a> TreeShaker<'a> {
  pub(crate) fn exec_formal_parameter(
    &mut self,
    node: &'a FormalParameter<'a>,
    arg: SymbolSource<'a>,
  ) {
    let data = self.load_data::<Data>(node);

    data.arg = arg;

    self.exec_binding_pattern(&node.pattern, BindingPatternSource::FormalParameter(node));
  }

  pub(crate) fn calc_formal_parameter(
    &self,
    node: &'a FormalParameter<'a>,
    symbol: SymbolId,
  ) -> Entity {
    let data = self.get_data::<Data>(node);

    let arg = self.calc_source(data.arg);

    todo!()
  }

  pub(crate) fn transform_formal_parameter(
    &self,
    node: FormalParameter<'a>,
  ) -> Vec<'a, FormalParameter<'a>> {
    let data = self.get_data::<Data>(&node);

    todo!();

    self.ast_builder.vec()
  }
}
