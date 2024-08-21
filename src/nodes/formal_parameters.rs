use crate::{
  nodes::binding_pattern::BindingPatternSource, symbol::arguments::ArgumentsEntity,
  transformer::Transformer, Analyzer,
};
use oxc::ast::ast::FormalParameters;

#[derive(Debug, Default, Clone)]
pub struct Data {}

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_formal_parameters(
    &mut self,
    node: &'a FormalParameters<'a>,
    args: ArgumentsEntity<'a>,
  ) {
    let resolved = args.resolve(node.items.len());

    for (param, arg) in node.items.iter().zip(resolved.0) {
      self.exec_formal_parameter(param, arg);
    }

    if let Some(rest) = &node.rest {
      self.exec_binding_rest_element(rest, BindingPatternSource::BindingRestElement(rest));
    }

    todo!()
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_formal_parameters(
    &mut self,
    node: FormalParameters<'a>,
  ) -> FormalParameters<'a> {
    let data = self.get_data::<Data>(&node);
    let FormalParameters { span, items, rest, kind, .. } = node;

    let mut transformed_items = self.ast_builder.vec();

    for param in items {
      transformed_items.append(&mut self.transform_formal_parameter(param));
    }

    let transformed_rest = match rest {
      Some(rest) => self.transform_binding_rest_element(rest.unbox()),
      None => None,
    };

    self.ast_builder.formal_parameters(span, kind, transformed_items, transformed_rest)
  }
}
