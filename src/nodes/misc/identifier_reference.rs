use crate::{analyzer::Analyzer, ast::AstKind2, entity::Entity, transformer::Transformer};
use oxc::{allocator, ast::ast::IdentifierReference};

impl<'a> Analyzer<'a> {
  pub fn exec_identifier_reference_read(
    &mut self,
    node: &'a IdentifierReference<'a>,
  ) -> Entity<'a> {
    let dep_id = AstKind2::IdentifierReference(node);
    let reference = self.semantic.symbols().get_reference(node.reference_id());
    let symbol = reference.symbol_id();

    if let Some(symbol) = symbol {
      // Known symbol
      if let Some(value) = self.read_symbol(symbol) {
        value
      } else {
        // TDZ
        self.refer_dep(dep_id);
        self.factory.unknown()
      }
    } else if node.name == "arguments" {
      // The `arguments` object
      let arguments_consumed = self.consume_arguments(None);
      self.call_scope_mut().need_consume_arguments = !arguments_consumed;
      self.factory.unknown()
    } else if let Some(global) = self.builtins.globals.get(node.name.as_str()) {
      // Known global
      *global
    } else {
      // Unknown global
      if self.config.unknown_global_side_effects {
        self.refer_dep(dep_id);
        self.refer_to_global();
        self.may_throw();
      }
      self.factory.unknown()
    }
  }

  pub fn exec_identifier_reference_write(
    &mut self,
    node: &'a IdentifierReference<'a>,
    value: Entity<'a>,
  ) {
    let dep_id = AstKind2::IdentifierReference(node);
    let value = self.factory.computed(value, dep_id);

    let reference = self.semantic.symbols().get_reference(node.reference_id());
    debug_assert!(reference.is_write());
    let symbol = reference.symbol_id();

    if let Some(symbol) = symbol {
      self.write_symbol(symbol, value);
    } else if self.builtins.globals.contains_key(node.name.as_str()) {
      self.add_diagnostic(
        "Should not write to builtin object, it may cause unexpected tree-shaking behavior",
      );
    } else {
      value.consume(self);
      self.refer_dep(dep_id);
      self.may_throw();
      self.refer_to_global();
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_identifier_reference(
    &self,
    node: &'a IdentifierReference<'a>,
    need_val: bool,
  ) -> Option<allocator::Box<'a, IdentifierReference<'a>>> {
    if need_val || self.is_referred(AstKind2::IdentifierReference(node)) {
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
