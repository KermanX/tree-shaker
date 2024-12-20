use super::{Entity, LiteralEntity};
use crate::analyzer::Analyzer;
use oxc::{
  ast::{ast::Expression, AstBuilder},
  span::Span,
};
use std::{cell::RefCell, mem, rc::Rc};

#[derive(Debug, Default)]
pub struct LiteralCollector<'a> {
  /// None if no literal is collected
  literal: Option<LiteralEntity<'a>>,
  /// Collected literal entities
  collected: Vec<Entity<'a>>,
  invalid: bool,
}

impl<'a> LiteralCollector<'a> {
  fn try_collect(
    &self,
    analyzer: &mut Analyzer<'a>,
    entity: Entity<'a>,
  ) -> Option<LiteralEntity<'a>> {
    if let Some(lit) = entity.get_literal(analyzer) {
      lit.can_build_expr(analyzer).then_some(lit)
    } else {
      None
    }
  }

  pub fn collect(&mut self, analyzer: &mut Analyzer<'a>, entity: Entity<'a>) -> Entity<'a> {
    if self.invalid {
      entity
    } else if let Some(literal) = self.try_collect(analyzer, entity) {
      if let Some(collected) = &self.literal {
        if collected != &literal {
          self.invalid = true;
          self.get_entity_on_invalid(entity, analyzer)
        } else {
          self.collected.push(entity);
          analyzer.factory.entity(literal)
        }
      } else {
        self.literal = Some(literal);
        self.collected.push(entity);
        analyzer.factory.entity(literal)
      }
    } else {
      self.invalid = true;
      self.get_entity_on_invalid(entity, analyzer)
    }
  }

  pub fn get_entity_on_invalid(
    &mut self,
    entity: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
  ) -> Entity<'a> {
    if self.collected.is_empty() {
      entity
    } else {
      analyzer.factory.collected(entity, Rc::new(RefCell::new(mem::take(&mut self.collected))))
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
