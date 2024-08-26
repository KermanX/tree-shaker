use std::{cell::RefCell, rc::Rc};

use super::{collected::CollectedEntity, entity::Entity, literal::LiteralEntity};
use oxc::{
  ast::{ast::Expression, AstBuilder},
  span::Span,
};

#[derive(Debug, Default)]
pub(crate) struct LiteralCollector<'a> {
  /// None if no literal is collected
  literal: Option<LiteralEntity<'a>>,
  /// Collected literal entities
  collected: Rc<RefCell<Vec<Entity<'a>>>>,
  invalid: bool,
}

impl<'a> LiteralCollector<'a> {
  pub(crate) fn collect(&mut self, entity: Entity<'a>) -> Entity<'a> {
    if self.invalid {
      self.get_entity_on_invalid(entity)
    } else if let Some(literal) = entity.get_literal() {
      if let Some(collected) = &self.literal {
        if collected != &literal {
          self.invalid = true;
          self.get_entity_on_invalid(entity)
        } else {
          self.collected.borrow_mut().push(entity);
          Rc::new(literal)
        }
      } else {
        self.literal = Some(literal);
        self.collected.borrow_mut().push(entity);
        Rc::new(literal)
      }
    } else {
      self.invalid = true;
      self.get_entity_on_invalid(entity)
    }
  }

  #[inline]
  pub(crate) fn get_entity_on_invalid(&self, entity: Entity<'a>) -> Entity<'a> {
    if self.collected.borrow().is_empty() {
      entity
    } else {
      CollectedEntity::new(entity, self.collected.clone())
    }
  }

  pub(crate) fn collected(&self) -> Option<LiteralEntity<'a>> {
    if self.invalid {
      None
    } else {
      self.literal
    }
  }

  pub(crate) fn build_expr(
    &self,
    ast_builder: &AstBuilder<'a>,
    span: Span,
  ) -> Option<Expression<'a>> {
    self.literal.as_ref().map(|literal| literal.build_expr(ast_builder, span))
  }
}
