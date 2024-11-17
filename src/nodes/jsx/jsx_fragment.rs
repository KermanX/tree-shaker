use crate::{analyzer::Analyzer, build_effect, entity::Entity, transformer::Transformer};
use oxc::ast::ast::{Expression, JSXFragment};

impl<'a> Analyzer<'a> {
  pub fn exec_jsx_fragment(&mut self, node: &'a JSXFragment<'a>) -> Entity<'a> {
    // already computed unknown
    self.exec_jsx_children(&node.children)
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_jsx_fragment(
    &self,
    node: &'a JSXFragment<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    if need_val {
      Some(
        self.ast_builder.expression_from_jsx_fragment(self.transform_jsx_fragment_need_val(node)),
      )
    } else {
      self.transform_jsx_fragment_effect_only(node)
    }
  }

  pub fn transform_jsx_fragment_effect_only(
    &self,
    node: &'a JSXFragment<'a>,
  ) -> Option<Expression<'a>> {
    let JSXFragment { span, children, .. } = node;

    build_effect!(self.ast_builder, *span, self.transform_jsx_children_effect_only(children),)
  }

  pub fn transform_jsx_fragment_need_val(&self, node: &'a JSXFragment<'a>) -> JSXFragment<'a> {
    let JSXFragment { span, opening_fragment, closing_fragment, children } = node;

    self.ast_builder.jsx_fragment(
      *span,
      self.clone_node(opening_fragment),
      self.clone_node(closing_fragment),
      self.transform_jsx_children_need_val(children),
    )
  }
}
