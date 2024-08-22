use super::binding_pattern::BindingPatternSource;
use crate::ast_type::AstType2;
use crate::{entity::Entity, transformer::Transformer, Analyzer};
use oxc::{ast::ast::BindingRestElement, semantic::SymbolId};
use rustc_hash::FxHashSet;

const AST_TYPE: AstType2 = AstType2::BindingRestElement;

#[derive(Debug, Default, Clone)]
pub struct Data {
  referred_symbols: FxHashSet<SymbolId>,
}

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_binding_rest_element(
    &mut self,
    node: &'a BindingRestElement<'a>,
    source: BindingPatternSource<'a>,
  ) {
  }

  pub(crate) fn calc_binding_rest_element(
    &self,
    node: &'a BindingRestElement<'a>,
    symbol: SymbolId,
  ) -> Option<Entity> {
    todo!()
  }

  pub(crate) fn refer_binding_rest_element(
    &mut self,
    node: &'a BindingRestElement<'a>,
    symbol: SymbolId,
  ) {
    todo!()
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_binding_rest_element(
    &self,
    node: BindingRestElement<'a>,
  ) -> Option<BindingRestElement<'a>> {
    let data = self.get_data::<Data>(AST_TYPE, &node);

    todo!()
  }
}
