use crate::{ast::DeclarationKind, transformer::Transformer, Analyzer};
use oxc::ast::ast::{
  ExportDefaultDeclaration, ExportDefaultDeclarationKind, ExportNamedDeclaration,
  ImportDeclaration, ImportDeclarationSpecifier, ImportDefaultSpecifier, ImportNamespaceSpecifier,
  ImportSpecifier, ModuleDeclaration, ModuleExportName,
};

impl<'a> Analyzer<'a> {
  pub fn declare_module_declaration(&mut self, node: &'a ModuleDeclaration<'a>) {
    match node {
      ModuleDeclaration::ImportDeclaration(node) => {
        if let Some(specifiers) = &node.specifiers {
          for specifier in specifiers {
            let local = specifier.local();
            self.declare_binding_identifier(local, false, DeclarationKind::Import);
            self.init_binding_identifier(local, Some(self.factory.unknown));
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
              let reference = self.semantic.symbols().get_reference(node.reference_id().unwrap());
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
          ExportDefaultDeclarationKind::FunctionDeclaration(node) => self.exec_function(node, true),
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

impl<'a> Transformer<'a> {
  pub fn transform_module_declaration(
    &self,
    node: &'a ModuleDeclaration<'a>,
  ) -> Option<ModuleDeclaration<'a>> {
    match node {
      ModuleDeclaration::ImportDeclaration(node) => {
        let ImportDeclaration { span, specifiers, source, with_clause, import_kind, .. } =
          node.as_ref();
        if let Some(specifiers) = specifiers {
          let mut transformed_specifiers = self.ast_builder.vec();
          for specifier in specifiers {
            let specifier = match specifier {
              ImportDeclarationSpecifier::ImportSpecifier(node) => {
                let ImportSpecifier { span, local, imported, import_kind, .. } = node.as_ref();
                self.transform_binding_identifier(local).map(|local| {
                  self.ast_builder.import_declaration_specifier_import_specifier(
                    *span,
                    imported.clone(),
                    local,
                    *import_kind,
                  )
                })
              }
              ImportDeclarationSpecifier::ImportDefaultSpecifier(node) => {
                let ImportDefaultSpecifier { span, local, .. } = node.as_ref();
                self.transform_binding_identifier(local).map(|local| {
                  self
                    .ast_builder
                    .import_declaration_specifier_import_default_specifier(*span, local)
                })
              }
              ImportDeclarationSpecifier::ImportNamespaceSpecifier(node) => {
                let ImportNamespaceSpecifier { span, local, .. } = node.as_ref();
                self.transform_binding_identifier(local).map(|local| {
                  self
                    .ast_builder
                    .import_declaration_specifier_import_namespace_specifier(*span, local)
                })
              }
            };
            if let Some(specifier) = specifier {
              transformed_specifiers.push(specifier);
            }
          }
          // FIXME: side effect in module
          if transformed_specifiers.is_empty() {
            None
          } else {
            Some(self.ast_builder.module_declaration_import_declaration(
              *span,
              Some(transformed_specifiers),
              source.clone(),
              self.clone_node(with_clause),
              *import_kind,
            ))
          }
        } else {
          Some(self.ast_builder.module_declaration_import_declaration(
            *span,
            None,
            source.clone(),
            self.clone_node(with_clause),
            *import_kind,
          ))
        }
      }
      ModuleDeclaration::ExportNamedDeclaration(node) => {
        if node.source.is_some() {
          // Re-exports. Nothing to do.
          return Some(
            self
              .ast_builder
              .module_declaration_from_export_named_declaration(self.clone_node(node)),
          );
        }
        let ExportNamedDeclaration {
          span,
          declaration,
          specifiers,
          source,
          export_kind,
          with_clause,
          ..
        } = node.as_ref();
        let declaration = declaration.as_ref().and_then(|d| self.transform_declaration(d));
        Some(self.ast_builder.module_declaration_export_named_declaration(
          *span,
          declaration,
          self.clone_node(specifiers),
          self.clone_node(source),
          *export_kind,
          self.clone_node(with_clause),
        ))
      }
      ModuleDeclaration::ExportDefaultDeclaration(node) => {
        let ExportDefaultDeclaration { span, declaration, exported, .. } = node.as_ref();
        let declaration = match declaration {
          ExportDefaultDeclarationKind::FunctionDeclaration(node) => {
            let function = self.transform_function(node, true).unwrap();
            self.ast_builder.export_default_declaration_kind_from_function(function)
          }
          ExportDefaultDeclarationKind::ClassDeclaration(node) => {
            let class = self.transform_class(node, true).unwrap();
            self.ast_builder.export_default_declaration_kind_from_class(class)
          }
          node => {
            let expression = self.transform_expression(node.to_expression(), true).unwrap();
            self.ast_builder.export_default_declaration_kind_expression(expression)
          }
        };
        Some(self.ast_builder.module_declaration_export_default_declaration(
          *span,
          declaration,
          exported.clone(),
        ))
      }
      ModuleDeclaration::ExportAllDeclaration(node) => {
        Some(self.ast_builder.module_declaration_from_export_all_declaration(self.clone_node(node)))
      }
      _ => unreachable!(),
    }
  }
}
