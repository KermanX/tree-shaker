use crate::{
  analyzer::Analyzer, ast::AstKind2, consumable::box_consumable, entity::Entity,
  transformer::Transformer,
};
use oxc::{allocator, ast::ast::IdentifierReference};

#[derive(Debug, Default, Clone)]
pub struct Data {
  has_effect: bool,
}

impl<'a> Analyzer<'a> {
  pub fn exec_identifier_reference_read(
    &mut self,
    node: &'a IdentifierReference<'a>,
  ) -> Entity<'a> {
    let reference = self.semantic.symbols().get_reference(node.reference_id());
    let symbol = reference.symbol_id();

    if let Some(symbol) = symbol {
      // Known symbol
      if let Some(value) = self.read_symbol(symbol) {
        value
      } else {
        // TDZ
        self.set_data(AstKind2::IdentifierReference(node), Data { has_effect: true });
        self.factory.unknown()
      }
    } else if node.name == "arguments" {
      // The `arguments` object
      let arguments_consumed = self.consume_arguments();
      self.call_scope_mut().need_consume_arguments = !arguments_consumed;
      self.factory.unknown()
    } else if let Some(global) = self.builtins.globals.get(node.name.as_str()) {
      // Known global
      *global
    } else {
      // Unknown global
      if self.is_inside_pure() {
        self.factory.computed_unknown(AstKind2::IdentifierReference(node))
      } else {
        if self.config.unknown_global_side_effects {
          self.set_data(AstKind2::IdentifierReference(node), Data { has_effect: true });
          self.refer_to_global();
          self.may_throw();
        }
        self.factory.unknown()
      }
    }
  }

  pub fn exec_identifier_reference_write(
    &mut self,
    node: &'a IdentifierReference<'a>,
    value: Entity<'a>,
  ) {
    let dep = box_consumable(AstKind2::IdentifierReference(node));
    let value = self.factory.computed(value, dep);

    let reference = self.semantic.symbols().get_reference(node.reference_id());
    assert!(reference.is_write());
    let symbol = reference.symbol_id();

    if let Some(symbol) = symbol {
      self.write_symbol(symbol, value);
    } else if self.builtins.globals.contains_key(node.name.as_str()) {
      self.add_diagnostic(
        "Should not write to builtin object, it may cause unexpected tree-shaking behavior",
      );
    } else {
      self.set_data(AstKind2::IdentifierReference(node), Data { has_effect: true });
      value.consume(self);
      self.may_throw();
      self.refer_to_global();
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_identifier_reference_read(
    &self,
    node: &'a IdentifierReference<'a>,
    need_val: bool,
  ) -> Option<allocator::Box<'a, IdentifierReference<'a>>> {
    let data = self.get_data::<Data>(AstKind2::IdentifierReference(node));

    self.transform_identifier_reference(node, data.has_effect || need_val)
  }

  pub fn transform_identifier_reference_write(
    &self,
    node: &'a IdentifierReference<'a>,
  ) -> Option<allocator::Box<'a, IdentifierReference<'a>>> {
    let data = self.get_data::<Data>(AstKind2::IdentifierReference(node));

    let referred = self.is_referred(AstKind2::IdentifierReference(node));

    self.transform_identifier_reference(node, data.has_effect || referred)
  }

  fn transform_identifier_reference(
    &self,
    node: &'a IdentifierReference<'a>,
    included: bool,
  ) -> Option<allocator::Box<'a, IdentifierReference<'a>>> {
    if included {
      let IdentifierReference { span, name, .. } = node;

      let reference = self.semantic.symbols().get_reference(node.reference_id());
      if let Some(symbol) = reference.symbol_id() {
        self.update_var_decl_state(symbol, false);
      }

      Some(self.ast_builder.alloc_identifier_reference(*span, name))
    } else {
      None
    }
  }
}
