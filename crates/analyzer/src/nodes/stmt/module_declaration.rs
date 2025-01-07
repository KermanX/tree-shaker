use crate::{host::Host, ast::DeclarationKind, Analyzer};
use oxc::ast::ast::{
  ExportDefaultDeclaration, ExportDefaultDeclarationKind, ExportNamedDeclaration,
  ImportDeclaration, ImportDeclarationSpecifier, ImportDefaultSpecifier, ImportNamespaceSpecifier,
  ImportSpecifier, ModuleDeclaration, ModuleExportName,
};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn declare_module_declaration(&mut self, node: &'a ModuleDeclaration<'a>) {
    match node {
      ModuleDeclaration::ImportDeclaration(node) => {
        if let Some(specifiers) = &node.specifiers {
          let name = node.source.value.as_str();
          let known = self.builtins.get_known_module(name);

          for specifier in specifiers {
            let value = if let Some(known) = known {
              match specifier {
                ImportDeclarationSpecifier::ImportDefaultSpecifier(_node) => known.default,
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(_node) => known.namespace,
                ImportDeclarationSpecifier::ImportSpecifier(node) => {
                  let key = self.factory.string(node.imported.name().as_str());
                  known.namespace.get_property(self, self.factory.empty_consumable, key)
                }
              }
            } else {
              self.builtins.factory.unknown()
            };

            let local = specifier.local();
            self.declare_binding_identifier(local, false, DeclarationKind::Import);

            self.init_binding_identifier(local, Some(value));
          }
        }
      }
      ModuleDeclaration::ExportNamedDeclaration(node) => {
        if node.source.is_some() {
          // Re-exports. Nothing to do.
          return;
        }
        if let Some(declaration) = &node.declaration {
          self.declare_declaration(declaration, true);
        }
        for specifier in &node.specifiers {
          match &specifier.local {
            ModuleExportName::IdentifierReference(node) => {
              let reference = self.semantic.symbols().get_reference(node.reference_id());
              if let Some(symbol) = reference.symbol_id() {
                self.named_exports.push(symbol);
              }
            }
            _ => unreachable!(),
          }
        }
      }
      ModuleDeclaration::ExportDefaultDeclaration(node) => {
        match &node.declaration {
          ExportDefaultDeclarationKind::FunctionDeclaration(node) => {
            if node.id.is_none() {
              // Patch `export default function(){}`
              return;
            }
            // Pass `exporting` as `false` because it is actually used as an expression
            self.declare_function(node, false);
          }
          ExportDefaultDeclarationKind::ClassDeclaration(node) => {
            if node.id.is_none() {
              // Patch `export default class{}`
              return;
            }
            // Pass `exporting` as `false` because it is actually used as an expression
            self.declare_class(node, false);
          }
          _expr => {}
        };
      }
      ModuleDeclaration::ExportAllDeclaration(_node) => {
        // Nothing to do
      }
      _ => unreachable!(),
    }
  }

  pub fn init_module_declaration(&mut self, node: &'a ModuleDeclaration<'a>) {
    match node {
      ModuleDeclaration::ImportDeclaration(_node) => {}
      ModuleDeclaration::ExportNamedDeclaration(node) => {
        if node.source.is_some() {
          // Re-exports. Nothing to do.
          return;
        }
        if let Some(declaration) = &node.declaration {
          self.init_declaration(declaration);
        }
      }
      ModuleDeclaration::ExportDefaultDeclaration(node) => {
        let value = match &node.declaration {
          ExportDefaultDeclarationKind::FunctionDeclaration(node) => self.exec_function(node),
          ExportDefaultDeclarationKind::ClassDeclaration(node) => {
            if node.id.is_none() {
              // Patch `export default class{}`
              self.exec_class(node)
            } else {
              self.init_class(node)
            }
          }
          node => self.exec_expression(node.to_expression()),
        };
        if self.default_export.is_some() {
          self.add_diagnostic("Duplicate default export");
        }
        self.default_export = Some(value);
      }
      ModuleDeclaration::ExportAllDeclaration(_node) => {
        // Nothing to do
      }
      _ => unreachable!(),
    }
  }
}

