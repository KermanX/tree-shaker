use super::binding_pattern::BindingPatternSource;
use crate::ast_type::AstType2;
use crate::{entity::Entity, transformer::Transformer, Analyzer};
use oxc::{ast::ast::VariableDeclarator, semantic::SymbolId, span::GetSpan};

const AST_TYPE: AstType2 = AstType2::VariableDeclarator;

#[derive(Debug, Default, Clone)]
pub struct Data {
  init_effect: bool,
  init_val: Entity,
}

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_variable_declarator(&mut self, node: &'a VariableDeclarator) -> bool {
    let (init_effect, init_val) = match &node.init {
      Some(init) => self.exec_expression(init),
      None => (false, Entity::Undefined),
    };

    self.exec_binding_pattern(
      &node.id,
      BindingPatternSource::VariableDeclarator(node),
      init_val.clone(),
    );

    self.set_data(AST_TYPE, node, Data { init_effect, init_val });

    init_effect
  }

  pub(crate) fn calc_variable_declarator(
    &self,
    node: &'a VariableDeclarator,
    symbol: SymbolId,
  ) -> Entity {
    self.calc_binding_pattern(&node.id, symbol).unwrap()
  }

  pub(crate) fn refer_variable_declarator(
    &mut self,
    node: &'a VariableDeclarator,
    symbol: SymbolId,
  ) {
    self.refer_binding_pattern(&node.id, symbol)
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_variable_declarator(
    &self,
    node: VariableDeclarator<'a>,
  ) -> Option<VariableDeclarator<'a>> {
    let data = self.get_data::<Data>(AST_TYPE, &node);

    let VariableDeclarator { span, kind, id, init, .. } = node;

    let id_span = id.span();
    let id = self.transform_binding_pattern(id);
    let init = init.and_then(|init| self.transform_expression(init, id.is_some()));

    if let Some(init) = init {
      Some(self.ast_builder.variable_declarator(
        span,
        kind,
        id.unwrap_or_else(|| self.new_unused_binding_pattern(id_span)),
        Some(init),
        false,
      ))
    } else if let Some(id) = id {
      Some(self.ast_builder.variable_declarator(span, kind, id, None, false))
    } else {
      None
    }
  }
}
