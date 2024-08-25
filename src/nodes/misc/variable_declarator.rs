use crate::ast::AstType2;
use crate::build_effect;
use crate::entity::collector::LiteralCollector;
use crate::entity::literal::LiteralEntity;
use crate::{transformer::Transformer, Analyzer};
use oxc::{ast::ast::VariableDeclarator, span::GetSpan};
use std::rc::Rc;

const AST_TYPE: AstType2 = AstType2::VariableDeclarator;

#[derive(Debug, Default)]
pub struct Data<'a> {
  collector: LiteralCollector<'a>,
}

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_variable_declarator(&mut self, node: &'a VariableDeclarator) {
    let init = match &node.init {
      Some(init) => self.exec_expression(init),
      None => Rc::new(LiteralEntity::Undefined),
    };

    let data = self.load_data::<Data>(AST_TYPE, node);
    data.collector.collect(&init);

    self.exec_binding_pattern(&node.id, init);
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

    let init = if id.is_some() {
      if let Some(literal) = data.collector.collected() {
        if let Some(init_effect) = init.and_then(|init| self.transform_expression(init, false)) {
          Some(
            build_effect!(&self.ast_builder, span, Some(init_effect); literal.build_expr(&self.ast_builder, span)),
          )
        } else if matches!(literal, LiteralEntity::Undefined) {
          None
        } else {
          Some(literal.build_expr(&self.ast_builder, span))
        }
      } else {
        init.and_then(|init| self.transform_expression(init, true))
      }
    } else {
      init.and_then(|init| self.transform_expression(init, false))
    };

    if let Some(init) = init {
      Some(self.ast_builder.variable_declarator(
        span,
        kind,
        id.unwrap_or_else(|| self.build_unused_binding_pattern(id_span)),
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
