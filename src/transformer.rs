use crate::{entity::Entity, utils::DataPlaceholder};
use oxc::{
  allocator::Allocator,
  ast::{
    ast::{Expression, NumberBase, Program, Statement},
    AstBuilder,
  },
  span::{GetSpan, SourceType, Span, SPAN},
};
use rustc_hash::FxHashMap;
use std::mem;

pub(crate) struct Transformer<'a> {
  allocator: &'a Allocator,
  pub ast_builder: AstBuilder<'a>,
  pub data: FxHashMap<Span, Box<DataPlaceholder<'a>>>,
}

impl<'a> Transformer<'a> {
  pub fn new(allocator: &'a Allocator, data: FxHashMap<Span, Box<DataPlaceholder<'a>>>) -> Self {
    Transformer { allocator, ast_builder: AstBuilder::new(allocator), data }
  }

  pub fn transform_program(&self, ast: &'a mut Program<'a>) -> Program<'a> {
    let Program { span, source_type, hashbang, directives, body: old_statements, .. } =
      mem::replace(
        ast,
        self.ast_builder.program(
          SPAN,
          SourceType::default(),
          None,
          self.ast_builder.vec(),
          self.ast_builder.vec(),
        ),
      );
    let mut new_statements = self.ast_builder.vec::<Statement>();
    for statement in old_statements {
      let new_statement = self.transform_statement(statement);
      if let Some(new_statement) = new_statement {
        new_statements.push(new_statement);
      }
    }
    self.ast_builder.program(span, source_type, hashbang, directives, new_statements)
  }

  pub(crate) fn entity_to_expression(&self, span: Span, value: &Entity) -> Option<Expression<'a>> {
    match value {
      Entity::StringLiteral(s) => Some(self.ast_builder.expression_string_literal(span, s.clone())),
      Entity::NumberLiteral(n) => Some(self.ast_builder.expression_numeric_literal(
        span,
        *n,
        n.to_string(),
        NumberBase::Decimal,
      )),
      Entity::BooleanLiteral(b) => Some(self.ast_builder.expression_boolean_literal(span, *b)),
      Entity::Null => Some(self.ast_builder.expression_null_literal(span)),
      Entity::Undefined => {
        Some(self.ast_builder.expression_identifier_reference(span, "undefined"))
      }
      _ => None,
    }
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn get_data_by_span<D: Default + 'a>(&self, span: Span) -> &'a D {
    let existing = self.data.get(&span);
    match existing {
      Some(boxed) => unsafe { mem::transmute(boxed.as_ref()) },
      None => self.allocator.alloc(D::default()),
    }
  }

  pub(crate) fn get_data<D: Default + 'a>(&self, node: &dyn GetSpan) -> &'a D {
    self.get_data_by_span(node.span())
  }
}
