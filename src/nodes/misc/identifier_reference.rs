use crate::ast::AstType2;
use crate::entity::dep::EntityDepNode;
use crate::entity::entity::Entity;
use crate::entity::forwarded::ForwardedEntity;
use crate::entity::unknown::UnknownEntity;
use crate::{transformer::Transformer, Analyzer};
use oxc::ast::ast::IdentifierReference;

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
      return global;
    }

    let reference = self.sematic.symbols().get_reference(node.reference_id().unwrap());
    let symbol = reference.symbol_id();

    self.set_data(AST_TYPE_READ, node, Data { resolvable: symbol.is_some() });

    if let Some(symbol) = symbol {
      self.get_symbol(&symbol)
    } else {
      // TODO: Handle globals
      self.refer_global_dep();
      UnknownEntity::new_unknown()
    }
  }

  pub fn exec_identifier_reference_export(&mut self, node: &'a IdentifierReference<'a>) {
    let reference = self.sematic.symbols().get_reference(node.reference_id().unwrap());
    debug_assert!(reference.is_read());
    let symbol = reference.symbol_id();
    self.exports.push(symbol.unwrap());
  }

  pub fn exec_identifier_reference_write(
    &mut self,
    node: &'a IdentifierReference<'a>,
    value: Entity<'a>,
  ) {
    let dep = self.new_entity_dep(EntityDepNode::IdentifierReference(node));
    let value = ForwardedEntity::new(value, dep);

    if self.builtins.globals.contains_key(node.name.as_str()) {
      // TODO: Throw warning
    }

    let reference = self.sematic.symbols().get_reference(node.reference_id().unwrap());
    debug_assert!(reference.is_write());
    let symbol = reference.symbol_id();

    self.set_data(AST_TYPE_WRITE, node, Data { resolvable: symbol.is_some() });

    if let Some(symbol) = symbol {
      self.set_symbol(&symbol, value);
    } else {
      value.consume_as_unknown(self);
      // TODO: Handle globals
      self.refer_global_dep();
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

    let referred = self.is_referred(EntityDepNode::IdentifierReference(node));
    (!data.resolvable || referred).then(|| self.clone_node(node))
  }
}
