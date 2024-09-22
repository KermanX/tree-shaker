use crate::{
  analyzer::Analyzer,
  ast::AstType2,
  entity::{entity::Entity, forwarded::ForwardedEntity, unknown::UnknownEntity},
  transformer::Transformer,
};
use oxc::ast::{ast::IdentifierReference, AstKind};

const AST_TYPE: AstType2 = AstType2::IdentifierReference;

#[derive(Debug, Default, Clone)]
pub struct Data {
  unknown: bool,
}

impl<'a> Analyzer<'a> {
  pub fn exec_identifier_reference_read(
    &mut self,
    node: &'a IdentifierReference<'a>,
  ) -> Entity<'a> {
    let reference = self.sematic.symbols().get_reference(node.reference_id().unwrap());
    let symbol = reference.symbol_id();

    if let Some(symbol) = symbol {
      self.read_symbol(&symbol)
    } else if node.name == "arguments" {
      let (args_entity, args_symbols) = self.call_scope().args.clone();
      args_entity.consume_as_unknown(self);
      for symbol in args_symbols {
        let old = self.read_symbol(&symbol);
        self.write_symbol(&symbol, UnknownEntity::new_unknown_with_deps(vec![old]));
      }
      UnknownEntity::new_unknown()
    } else if let Some(global) = self.builtins.globals.get(node.name.as_str()).cloned() {
      self.may_throw();
      return global;
    } else {
      self.set_data(AST_TYPE, node, Data { unknown: true });
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

    let reference = self.sematic.symbols().get_reference(node.reference_id().unwrap());
    // Upstream bug
    // debug_assert!(reference.is_write());
    let symbol = reference.symbol_id();

    if let Some(symbol) = symbol {
      self.write_symbol(&symbol, value);
    } else if self.builtins.globals.contains_key(node.name.as_str()) {
      // TODO: Throw warning
    } else {
      self.set_data(AST_TYPE, node, Data { unknown: true });
      value.consume_as_unknown(self);
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

    (data.unknown || need_val).then(|| self.clone_node(node))
  }

  pub fn transform_identifier_reference_write(
    &self,
    node: &'a IdentifierReference<'a>,
  ) -> Option<IdentifierReference<'a>> {
    let data = self.get_data::<Data>(AST_TYPE, node);

    let referred = self.is_referred(AstKind::IdentifierReference(node));

    (data.unknown || referred).then(|| self.clone_node(node))
  }
}
