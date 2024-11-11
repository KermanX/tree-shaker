use crate::{analyzer::Analyzer, entity::Entity, transformer::Transformer};
use oxc::ast::ast::{Expression, JSXMemberExpressionObject};

impl<'a> Analyzer<'a> {
  pub fn exec_jsx_member_expression_object(
    &mut self,
    node: &'a JSXMemberExpressionObject<'a>,
  ) -> Entity<'a> {
    match node {
      JSXMemberExpressionObject::IdentifierReference(node) => {
        self.exec_identifier_reference_read(node)
      }
      JSXMemberExpressionObject::MemberExpression(node) => self.exec_jsx_member_expression(node),
      JSXMemberExpressionObject::ThisExpression(node) => self.exec_this_expression(node),
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_jsx_member_expression_object_effect_only(
    &self,
    node: &'a JSXMemberExpressionObject<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    match node {
      JSXMemberExpressionObject::IdentifierReference(node) => self
        .transform_identifier_reference_read(node, need_val)
        .map(|id| self.ast_builder.expression_from_identifier_reference(id)),
      JSXMemberExpressionObject::MemberExpression(node) => {
        self.transform_jsx_member_expression_effect_only(node, need_val)
      }
      JSXMemberExpressionObject::ThisExpression(node) => {
        need_val.then_some(self.ast_builder.expression_this(node.span))
      }
    }
  }

  pub fn transform_jsx_member_expression_object_need_val(
    &self,
    node: &'a JSXMemberExpressionObject<'a>,
  ) -> JSXMemberExpressionObject<'a> {
    match node {
      JSXMemberExpressionObject::IdentifierReference(node) => {
        self.ast_builder.jsx_member_expression_object_from_identifier_reference(
          self.transform_identifier_reference_read(node, true).unwrap(),
        )
      }
      JSXMemberExpressionObject::MemberExpression(node) => {
        self.ast_builder.jsx_member_expression_object_from_jsx_member_expression(
          self.transform_jsx_member_expression_need_val(node),
        )
      }
      JSXMemberExpressionObject::ThisExpression(node) => {
        self.ast_builder.jsx_member_expression_object_from_this_expression(self.clone_node(node))
      }
    }
  }
}
