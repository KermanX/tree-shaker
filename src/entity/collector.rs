use super::{CollectedEntity, Entity, LiteralEntity};
use crate::analyzer::Analyzer;
use oxc::{
  ast::{ast::Expression, AstBuilder},
  span::Span,
};
use std::{cell::RefCell, rc::Rc};

#[derive(Debug)]
pub struct LiteralCollector<'a> {
  try_collect: fn(&Analyzer<'a>, &Entity<'a>) -> Option<LiteralEntity<'a>>,

  /// None if no literal is collected
  literal: Option<LiteralEntity<'a>>,
  /// Collected literal entities
  collected: Rc<RefCell<Vec<Entity<'a>>>>,
  invalid: bool,
}

impl<'a> LiteralCollector<'a> {
  pub fn new_expr_collector() -> Self {
    Self {
      try_collect: |analyzer, entity| {
        entity.get_literal().and_then(|lit| lit.can_build_expr(analyzer).then_some(lit))
      },
      literal: None,
      collected: Rc::new(RefCell::new(Vec::new())),
      invalid: false,
    }
  }

  pub fn new_property_key_collector() -> Self {
    Self {
      try_collect: |analyzer, entity| match entity.get_literal() {
        Some(lit @ LiteralEntity::String(str))
          if str.len() <= analyzer.config.max_simple_string_length
            && analyzer.config.static_property_key_regex.is_match(str) =>
        {
          Some(lit)
        }
        _ => None,
      },
      literal: None,
      collected: Rc::new(RefCell::new(Vec::new())),
      invalid: false,
    }
  }

  pub fn collect(&mut self, analyzer: &Analyzer<'a>, entity: Entity<'a>) -> Entity<'a> {
    if self.invalid {
      entity
    } else if let Some(literal) = (self.try_collect)(analyzer, &entity) {
      if let Some(collected) = &self.literal {
        if collected != &literal {
          self.invalid = true;
          self.get_entity_on_invalid(entity)
        } else {
          self.collected.borrow_mut().push(entity);
          Entity::new(literal)
        }
      } else {
        self.literal = Some(literal);
        self.collected.borrow_mut().push(entity);
        Entity::new(literal)
      }
    } else {
      self.invalid = true;
      self.get_entity_on_invalid(entity)
    }
  }

  #[inline]
  pub fn get_entity_on_invalid(&self, entity: Entity<'a>) -> Entity<'a> {
    if self.collected.borrow().is_empty() {
      entity
    } else {
      CollectedEntity::new(entity, self.collected.clone())
    }
  }

  pub fn collected(&self) -> Option<LiteralEntity<'a>> {
    if self.invalid {
      None
    } else {
      self.literal
    }
  }

  pub fn build_expr(&self, ast_builder: &AstBuilder<'a>, span: Span) -> Option<Expression<'a>> {
    if self.invalid {
      None
    } else {
      self.literal.as_ref().map(|literal| literal.build_expr(ast_builder, span))
    }
  }
}
