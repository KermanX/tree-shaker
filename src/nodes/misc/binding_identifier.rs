use crate::{
  analyzer::Analyzer, ast::DeclarationKind, entity::entity::Entity, transformer::Transformer,
};
use oxc::ast::{ast::BindingIdentifier, AstKind};

impl<'a> Analyzer<'a> {
  pub fn declare_binding_identifier(
    &mut self,
    node: &'a BindingIdentifier<'a>,
    exporting: bool,
    kind: DeclarationKind,
  ) {
    let symbol = node.symbol_id.get().unwrap();
    let dep = AstKind::BindingIdentifier(node);
    self.declare_symbol(symbol, dep, exporting, kind, None);
  }

  pub fn init_binding_identifier(
    &mut self,
    node: &'a BindingIdentifier<'a>,
    init: Option<Entity<'a>>,
  ) {
    let symbol = node.symbol_id.get().unwrap();
    let dep = AstKind::BindingIdentifier(node);
    self.init_symbol(symbol, init, dep.into());
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_binding_identifier(
    &self,
    node: &'a BindingIdentifier<'a>,
  ) -> Option<BindingIdentifier<'a>> {
    let symbol = node.symbol_id.get().unwrap();
    let call_stack = self.call_stack.borrow();
    let key = call_stack.last().unwrap();
    if let Some(var_decls) = self.var_decls.borrow_mut().get_mut(key) {
      var_decls.remove(&symbol);
    }

    let referred = self.is_referred(AstKind::BindingIdentifier(&node));
    referred.then(|| self.clone_node(node))
  }
}
