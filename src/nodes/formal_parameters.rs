use crate::{
  context::Context,
  entity::{arguments::ArgumentsEntity, Entity},
  symbol::SymbolSource,
  TreeShaker,
};
use oxc::ast::ast::FormalParameters;

#[derive(Debug, Default, Clone)]
pub struct Data {}

impl<'a> TreeShaker<'a> {
  pub(crate) fn exec_formal_parameters(
    &mut self,
    node: &'a FormalParameters,
    args: ArgumentsEntity,
  ) {
    let data = self.load_data::<Data>(node);

    let resolved = args.resolve(node.items.len());

    for (param, arg) in node.items.iter().zip(resolved.0) {
      self.exec_formal_parameter(param, arg);
    }

    if let Some(rest) = &node.rest {
      self.exec_binding_rest_element(rest, SymbolSource::BindingRestElement(rest, resolved.1));
    }

    todo!()
  }
}
