use crate::{ast_type::AstType2, entity::Entity, utils::{DataPlaceholder, ExtraData}};
use oxc::{
  allocator::Allocator,
  ast::{
    ast::{
      BindingPattern, Expression, NumberBase, Program, Statement, TSTypeAnnotation, UnaryOperator,
    },
    AstBuilder,
  },
  span::{GetSpan, SourceType, Span, SPAN},
};
use std::{
  mem,
  sync::atomic::{AtomicUsize, Ordering},
};

pub(crate) struct Transformer<'a> {
  allocator: &'a Allocator,
  pub ast_builder: AstBuilder<'a>,
  pub data: ExtraData<'a>,
}

impl<'a> Transformer<'a> {
  pub fn new(allocator: &'a Allocator, data: ExtraData<'a>) -> Self {
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

static COUNTER: AtomicUsize = AtomicUsize::new(0);

impl<'a> Transformer<'a> {
  pub(crate) fn new_unused_binding_pattern(&self, span: Span) -> BindingPattern<'a> {
    let name = format!("__unused_{}", COUNTER.fetch_add(1, Ordering::Relaxed));
    self.ast_builder.binding_pattern(
      self.ast_builder.binding_pattern_kind_binding_identifier(span, name),
      None::<TSTypeAnnotation>,
      false,
    )
  }

  pub(crate) fn negate_expression(&self, expression: Expression<'a>) -> Expression<'a> {
    self.ast_builder.expression_unary(expression.span(), UnaryOperator::LogicalNot, expression)
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn get_data_by_span<D: Default + 'a>(&self, ast_type: AstType2, span: Span) -> &'a D {
    let existing = self.data.get(&ast_type).and_then(|map| map.get(&span));
    match existing {
      Some(boxed) => unsafe { mem::transmute::<&DataPlaceholder<'_>, &D>(boxed.as_ref()) },
      None => self.allocator.alloc(D::default()),
    }
  }

  pub(crate) fn get_data<D: Default + 'a>(&self, ast_type: AstType2, node: &dyn GetSpan) -> &'a D {
    self.get_data_by_span(ast_type, node.span())
  }
}
