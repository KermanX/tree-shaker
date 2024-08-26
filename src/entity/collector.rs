use std::rc::Rc;

use super::{entity::Entity, literal::LiteralEntity};
use oxc::{
  ast::{ast::Expression, AstBuilder},
  span::Span,
};

#[derive(Debug, Default)]
pub(crate) struct LiteralCollector<'a> {
  /// None if no literal is collected
  collected: Option<LiteralEntity<'a>>,
  invalid: bool,
}

impl<'a> LiteralCollector<'a> {
  pub(crate) fn collect(&mut self, entity: Entity<'a>) -> Entity<'a> {
    if self.invalid {
      return entity;
    }
    if let Some(literal) = entity.get_literal() {
      if let Some(collected) = &self.collected {
        if collected != &literal {
          self.invalid = true;
          return entity;
        } else {
          return Rc::new(literal);
        }
      } else {
        self.collected = Some(literal);
        return Rc::new(literal);
      }
    } else {
      self.invalid = true;
      return entity;
    }
  }

  pub(crate) fn collected(&self) -> Option<LiteralEntity<'a>> {
    if self.invalid {
      None
    } else {
      assert!(self.collected.is_some());
      self.collected
    }
  }

  pub(crate) fn build_expr(
    &self,
    ast_builder: &AstBuilder<'a>,
    span: Span,
  ) -> Option<Expression<'a>> {
    self.collected.as_ref().map(|literal| literal.build_expr(ast_builder, span))
  }
}
