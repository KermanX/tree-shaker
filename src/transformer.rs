use crate::{
  analyzer::Analyzer,
  ast::AstType2,
  data::{DataPlaceholder, ExtraData, ReferredNodes},
  entity::dep::EntityDepNode,
};
use oxc::{
  allocator::Allocator,
  ast::{
    ast::{BindingPattern, Expression, NumberBase, Program, TSTypeAnnotation, UnaryOperator},
    AstBuilder,
  },
  span::{GetSpan, Span, SPAN},
};
use std::{
  hash::{DefaultHasher, Hasher},
  mem,
};

pub(crate) struct Transformer<'a> {
  pub allocator: &'a Allocator,
  pub ast_builder: AstBuilder<'a>,
  pub data: ExtraData<'a>,
  pub referred_nodes: ReferredNodes<'a>,
}

impl<'a> Transformer<'a> {
  pub fn new(analyzer: Analyzer<'a>) -> Self {
    let Analyzer { allocator, data, referred_nodes, .. } = analyzer;
    Transformer { allocator, ast_builder: AstBuilder::new(allocator), data, referred_nodes }
  }

  pub fn transform_program(&mut self, ast: Program<'a>) -> Program<'a> {
    let Program { span, source_type, hashbang, directives, body, .. } = ast;
    let mut transformed_body = self.ast_builder.vec();
    for stmt in body {
      let new_statement = self.transform_statement(stmt);
      if let Some(transformed) = new_statement {
        transformed_body.push(transformed);
      }
    }
    self.ast_builder.program(span, source_type, hashbang, directives, transformed_body)
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn build_unused_binding_pattern(&self, span: Span) -> BindingPattern<'a> {
    let mut hasher = DefaultHasher::new();
    hasher.write_u32(span.start);
    hasher.write_u32(span.end);
    let name = format!("__unused_{:04X}", hasher.finish() % 0xFFFF);
    self.ast_builder.binding_pattern(
      self.ast_builder.binding_pattern_kind_binding_identifier(span, name),
      None::<TSTypeAnnotation>,
      false,
    )
  }

  pub(crate) fn build_unused_expression(&self, span: Span) -> Expression<'a> {
    self.ast_builder.expression_numeric_literal(span, 0.0f64, "0", NumberBase::Decimal)
  }

  pub(crate) fn build_unused_iterable(&self, span: Span, length: usize) -> Expression<'a> {
    let mut elements = self.ast_builder.vec();
    for _ in 0..length {
      elements.push(
        self.ast_builder.array_expression_element_expression(self.build_unused_expression(SPAN)),
      );
    }
    self.ast_builder.expression_array(span, elements, None)
  }

  pub(crate) fn build_undefined(&self, span: Span) -> Expression<'a> {
    self.ast_builder.expression_identifier_reference(span, "undefined")
  }

  pub(crate) fn build_negate_expression(&self, expression: Expression<'a>) -> Expression<'a> {
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

impl<'a> Transformer<'a> {
  pub(crate) fn is_referred(&self, node: EntityDepNode<'a>) -> bool {
    self.referred_nodes.contains(&node)
  }
}
