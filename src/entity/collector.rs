use super::{CollectedEntity, Entity, LiteralEntity};
use crate::{analyzer::Analyzer, TreeShakeConfig};
use oxc::{
  ast::{ast::Expression, AstBuilder},
  span::Span,
};
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Default)]
pub struct LiteralCollector<'a> {
  /// None if no literal is collected
  literal: Option<LiteralEntity<'a>>,
  /// Collected literal entities
  collected: Rc<RefCell<Vec<Entity<'a>>>>,
  invalid: bool,
}

impl<'a> LiteralCollector<'a> {
  pub fn collect(&mut self, analyzer: &Analyzer<'a>, entity: Entity<'a>) -> Entity<'a> {
    if self.invalid {
      entity
    } else if let Some(literal) =
      entity.get_literal().and_then(|lit| lit.can_build_expr(analyzer).then_some(lit))
    {
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

  pub fn collected_property_key(&self, config: &TreeShakeConfig) -> Option<(bool, &'a str)> {
    let collected = self.collected()?;
    let str = collected.to_string();
    if str.len() > config.max_simple_string_length {
      return None;
    }
    let r#static = !config.static_property_key_regex.is_match(str);
    Some((r#static, str))
  }

  pub fn build_expr(&self, ast_builder: &AstBuilder<'a>, span: Span) -> Option<Expression<'a>> {
    if self.invalid {
      None
    } else {
      self.literal.as_ref().map(|literal| literal.build_expr(ast_builder, span))
    }
  }
}
