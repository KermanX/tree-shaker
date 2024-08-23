use super::binding_pattern::BindingPatternSource;
use crate::ast::AstType2;
use crate::{entity::Entity, transformer::Transformer, Analyzer};
use oxc::{ast::ast::BindingRestElement, semantic::SymbolId};

const AST_TYPE: AstType2 = AstType2::BindingRestElement;

#[derive(Debug, Default, Clone)]
pub struct Data {}

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_binding_rest_element(
    &mut self,
    node: &'a BindingRestElement<'a>,
    source: BindingPatternSource<'a>,
    init_val: Entity,
  ) -> bool {
    self.exec_binding_pattern(&node.argument, source, init_val)
  }

  pub(crate) fn calc_binding_rest_element(
    &self,
    node: &'a BindingRestElement<'a>,
    symbol: SymbolId,
  ) -> Option<Entity> {
    self.calc_binding_pattern(&node.argument, symbol)
  }

  pub(crate) fn refer_binding_rest_element(
    &mut self,
    node: &'a BindingRestElement<'a>,
    symbol: SymbolId,
  ) {
    self.refer_binding_pattern(&node.argument, symbol)
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_binding_rest_element(
    &self,
    node: BindingRestElement<'a>,
  ) -> Option<BindingRestElement<'a>> {
    let BindingRestElement { span, argument, .. } = node;

    self
      .transform_binding_pattern(argument)
      .map(|argument| self.ast_builder.binding_rest_element(span, argument))
  }
}
