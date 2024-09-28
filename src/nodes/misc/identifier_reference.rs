use crate::{
  analyzer::Analyzer,
  ast::AstType2,
  entity::{Entity, ForwardedEntity, UnknownEntity},
  transformer::Transformer,
};
use oxc::ast::{ast::IdentifierReference, AstKind};

const AST_TYPE: AstType2 = AstType2::IdentifierReference;

#[derive(Debug, Default, Clone)]
pub struct Data {
  has_effect: bool,
}

impl<'a> Analyzer<'a> {
  pub fn exec_identifier_reference_read(
    &mut self,
    node: &'a IdentifierReference<'a>,
  ) -> Entity<'a> {
    let reference = self.semantic.symbols().get_reference(node.reference_id().unwrap());
    let symbol = reference.symbol_id();

    if let Some(symbol) = symbol {
      if let Some(value) = self.read_symbol(&symbol) {
        value
      } else {
        self.set_data(AST_TYPE, node, Data { has_effect: true });
        UnknownEntity::new_unknown()
      }
    } else if node.name == "arguments" {
      let arguments_consumed = self.consume_arguments(None);
      self.call_scope_mut().need_consume_arguments = !arguments_consumed;
      UnknownEntity::new_unknown()
    } else if let Some(global) = self.builtins.globals.get(node.name.as_str()).cloned() {
      global
    } else {
      self.set_data(AST_TYPE, node, Data { has_effect: true });
      self.refer_global();
      self.may_throw();
      UnknownEntity::new_unknown()
    }
  }

  pub fn exec_identifier_reference_write(
    &mut self,
    node: &'a IdentifierReference<'a>,
    value: Entity<'a>,
  ) {
    let dep = AstKind::IdentifierReference(node);
    let value = ForwardedEntity::new(value, dep);

    let reference = self.semantic.symbols().get_reference(node.reference_id().unwrap());
    // Upstream bug
    // debug_assert!(reference.is_write());
    let symbol = reference.symbol_id();

    if let Some(symbol) = symbol {
      self.write_symbol(&symbol, value);
    } else if self.builtins.globals.contains_key(node.name.as_str()) {
      // TODO: Throw warning
    } else {
      self.set_data(AST_TYPE, node, Data { has_effect: true });
      value.consume(self);
      self.may_throw();
      self.refer_global();
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_identifier_reference_read(
    &self,
    node: &'a IdentifierReference<'a>,
    need_val: bool,
  ) -> Option<IdentifierReference<'a>> {
    let data = self.get_data::<Data>(AST_TYPE, node);

    (data.has_effect || need_val).then(|| self.clone_node(node))
  }

  pub fn transform_identifier_reference_write(
    &self,
    node: &'a IdentifierReference<'a>,
  ) -> Option<IdentifierReference<'a>> {
    let data = self.get_data::<Data>(AST_TYPE, node);

    let referred = self.is_referred(AstKind::IdentifierReference(node));

    (data.has_effect || referred).then(|| self.clone_node(node))
  }
}
