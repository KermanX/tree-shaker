use super::{Entity, LiteralEntity};
use crate::{analyzer::Analyzer, mangling::MangleAtom};
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
  mangle_atom: Option<MangleAtom>,
}

impl<'a> LiteralCollector<'a> {
  fn try_collect(&self, analyzer: &Analyzer<'a>, entity: Entity<'a>) -> Option<LiteralEntity<'a>> {
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
      if let Some(collected) = self.literal {
        if collected == literal {
          self.on_collectable(analyzer, entity, literal)
        } else {
          self.on_invalid(analyzer, entity)
        }
      } else {
        self.literal = Some(literal);
        self.on_collectable(analyzer, entity, literal)
      }
    } else {
      self.on_invalid(analyzer, entity)
    }
  }

  fn on_invalid(&mut self, analyzer: &mut Analyzer<'a>, entity: Entity<'a>) -> Entity<'a> {
    self.invalid = true;
    if self.collected.is_empty() {
      entity
    } else {
      let val =
        analyzer.factory.collected(entity, Rc::new(RefCell::new(mem::take(&mut self.collected))));
      if let Some(mangle_atom) = mem::take(&mut self.mangle_atom) {
        analyzer.factory.computed(val, mangle_atom)
      } else {
        val
      }
    }
  }

  fn on_collectable(
    &mut self,
    analyzer: &mut Analyzer<'a>,
    entity: Entity<'a>,
    literal: LiteralEntity<'a>,
  ) -> Entity<'a> {
    self.collected.push(entity);
    literal.with_mangle_atom(analyzer, &mut self.mangle_atom)
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
