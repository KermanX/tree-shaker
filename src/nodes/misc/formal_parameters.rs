use crate::ast::AstType2;
use crate::{symbol::arguments::ArgumentsSource, transformer::Transformer, Analyzer};
use oxc::ast::ast::FormalParameters;

use super::binding_pattern::BindingPatternSource;

const AST_TYPE: AstType2 = AstType2::FormalParameters;

#[derive(Debug, Default, Clone)]
pub struct Data {}

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_formal_parameters(
    &mut self,
    node: &'a FormalParameters<'a>,
    args: &'a dyn ArgumentsSource<'a>,
  ) {
    let resolved = args.resolve(node.items.len());

    for (param, arg) in node.items.iter().zip(resolved.0) {
      self.exec_formal_parameter(param, arg);
    }

    if let Some(rest) = &node.rest {
      let init_val = self.calc_source(resolved.1);
      self.exec_binding_rest_element(
        rest,
        BindingPatternSource::BindingRestElement(rest),
        init_val,
      );
    }
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_formal_parameters(
    &mut self,
    node: FormalParameters<'a>,
  ) -> FormalParameters<'a> {
    let data = self.get_data::<Data>(AST_TYPE, &node);
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
