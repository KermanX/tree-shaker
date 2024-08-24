use super::{entity::Entity, literal::LiteralEntity};
use oxc::{
  ast::{
    ast::{BigintBase, Expression, NumberBase},
    AstBuilder,
  },
  span::Span,
};

#[derive(Debug, Default)]
pub(crate) struct LiteralCollector<'a> {
  /// None if no literal is collected
  collected: Option<LiteralEntity<'a>>,
  invalid: bool,
}

impl<'a> LiteralCollector<'a> {
  pub(crate) fn collect(&mut self, entity: &Entity<'a>) {
    if self.invalid {
      return;
    }
    if let Some(literal) = entity.get_literal() {
      if let Some(collected) = &self.collected {
        if collected != &literal {
          self.invalid = true;
        }
      } else {
        self.collected = Some(literal);
      }
    } else {
      self.invalid = true;
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
    self.collected.as_ref().map(|literal| match literal {
      LiteralEntity::String(value) => ast_builder.expression_string_literal(span, *value),
      LiteralEntity::Number(value, raw) => {
        ast_builder.expression_numeric_literal(span, value.0, *raw, NumberBase::Decimal)
      }
      LiteralEntity::BigInt(value) => {
        ast_builder.expression_big_int_literal(span, *value, BigintBase::Decimal)
      }
      LiteralEntity::Boolean(value) => ast_builder.expression_boolean_literal(span, *value),
      LiteralEntity::Symbol(value) => todo!(),
      LiteralEntity::Null => ast_builder.expression_null_literal(span),
      LiteralEntity::Undefined => ast_builder.expression_identifier_reference(span, "undefined"),
    })
  }
}
