use crate::ast::DeclarationKind;
use crate::entity::entity::Entity;
use crate::{transformer::Transformer, Analyzer};
use oxc::ast::ast::{FormalParameter, FormalParameters};

impl<'a> Analyzer<'a> {
  pub fn exec_formal_parameters(&mut self, node: &'a FormalParameters<'a>, args: Entity<'a>) {
    let (elements_init, rest_init) = args.get_to_array(node.items.len());

    for (param, init) in node.items.iter().zip(elements_init) {
      self.declare_binding_pattern(&param.pattern, false, DeclarationKind::Parameter);
      self.exec_binding_pattern(&param.pattern, init);
    }

    if let Some(rest) = &node.rest {
      self.declare_binding_rest_element(rest, false, DeclarationKind::Parameter);
      self.init_binding_rest_element(rest, rest_init);
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_formal_parameters(
    &self,
    node: &'a FormalParameters<'a>,
  ) -> FormalParameters<'a> {
    let FormalParameters { span, items, rest, kind, .. } = node;

    let mut transformed_items = self.ast_builder.vec();

    for param in items {
      let FormalParameter { span, decorators, pattern, .. } = param;
      let pattern = self.transform_binding_pattern(pattern);
      transformed_items.push(self.ast_builder.formal_parameter(
        *span,
        self.clone_node(decorators),
        pattern.unwrap_or_else(|| self.build_unused_binding_pattern(*span)),
        None,
        false,
        false,
      ));
    }

    let transformed_rest = match rest {
      Some(rest) => self.transform_binding_rest_element(rest),
      None => None,
    };

    self.ast_builder.formal_parameters(*span, *kind, transformed_items, transformed_rest)
  }
}
