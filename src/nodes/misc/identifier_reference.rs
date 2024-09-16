use crate::{
  analyzer::Analyzer,
  ast::AstType2,
  entity::{entity::Entity, forwarded::ForwardedEntity, unknown::UnknownEntity},
  transformer::Transformer,
};
use oxc::ast::{ast::IdentifierReference, AstKind};

const AST_TYPE_READ: AstType2 = AstType2::IdentifierReferenceRead;
const AST_TYPE_WRITE: AstType2 = AstType2::IdentifierReferenceWrite;

#[derive(Debug, Default, Clone)]
pub struct Data {
  resolvable: bool,
}

impl<'a> Analyzer<'a> {
  pub fn exec_identifier_reference_read(
    &mut self,
    node: &'a IdentifierReference<'a>,
  ) -> Entity<'a> {
    if let Some(global) = self.builtins.globals.get(node.name.as_str()).cloned() {
      self.set_data(AST_TYPE_READ, node, Data { resolvable: true });
      self.may_throw();
      return global;
    }

    let reference = self.sematic.symbols().get_reference(node.reference_id().unwrap());
    let symbol = reference.symbol_id();

    self.set_data(AST_TYPE_READ, node, Data { resolvable: symbol.is_some() });

    if let Some(symbol) = symbol {
      self.read_symbol(&symbol)
    } else {
      // TODO: Handle globals
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

    if self.builtins.globals.contains_key(node.name.as_str()) {
      // TODO: Throw warning
    }

    let reference = self.sematic.symbols().get_reference(node.reference_id().unwrap());
    debug_assert!(reference.is_write());
    let symbol = reference.symbol_id();

    self.set_data(AST_TYPE_WRITE, node, Data { resolvable: symbol.is_some() });

    if let Some(symbol) = symbol {
      self.write_symbol(&symbol, value);
    } else {
      value.consume_as_unknown(self);
      // TODO: Handle globals
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
    let data = self.get_data::<Data>(AST_TYPE_READ, node);

    (!data.resolvable || need_val).then(|| self.clone_node(node))
  }

  pub fn transform_identifier_reference_write(
    &self,
    node: &'a IdentifierReference<'a>,
  ) -> Option<IdentifierReference<'a>> {
    let data = self.get_data::<Data>(AST_TYPE_WRITE, node);

    let referred = self.is_referred(AstKind::IdentifierReference(node));
    (!data.resolvable || referred).then(|| self.clone_node(node))
  }
}
