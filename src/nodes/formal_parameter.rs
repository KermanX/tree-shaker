use crate::{entity::Entity, symbol::SymbolSource, transformer::Transformer, Analyzer};
use oxc::{allocator::Vec, ast::ast::FormalParameter, semantic::SymbolId};

use super::binding_pattern::BindingPatternSource;

#[derive(Debug, Default, Clone)]
pub struct Data<'a> {
  arg: SymbolSource<'a>,
}

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_formal_parameter(
    &mut self,
    node: &'a FormalParameter<'a>,
    arg: SymbolSource<'a>,
  ) {
    let init_val = self.calc_source(arg);
    self.exec_binding_pattern(&node.pattern, BindingPatternSource::FormalParameter(node), init_val);
    self.set_data(node, Data { arg });
  }

  pub(crate) fn calc_formal_parameter(
    &self,
    node: &'a FormalParameter<'a>,
    symbol: SymbolId,
  ) -> Entity {
    self.calc_binding_pattern(&node.pattern, symbol).unwrap()
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_formal_parameter(
    &self,
    node: FormalParameter<'a>,
  ) -> Vec<'a, FormalParameter<'a>> {
    let data = self.get_data::<Data>(&node);

    todo!();

    self.ast_builder.vec()
  }
}
