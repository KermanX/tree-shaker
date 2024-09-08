use crate::{entity::unknown::UnknownEntity, transformer::Transformer, Analyzer};
use oxc::ast::ast::{
  ExportDefaultDeclaration, ExportDefaultDeclarationKind, ExportNamedDeclaration,
  ImportDeclaration, ImportDeclarationSpecifier, ImportDefaultSpecifier, ImportNamespaceSpecifier,
  ImportSpecifier, ModuleDeclaration, ModuleExportName,
};

impl<'a> Analyzer<'a> {
  pub fn exec_module_declaration(&mut self, node: &'a ModuleDeclaration<'a>) {
    match node {
      ModuleDeclaration::ImportDeclaration(node) => {
        if let Some(specifiers) = &node.specifiers {
          for specifier in specifiers {
            self.exec_binding_identifier(
              specifier.local(),
              UnknownEntity::new_unknown(),
              false,
              false,
            )
          }
        }
      }
      ModuleDeclaration::ExportNamedDeclaration(node) => {
        if node.source.is_some() {
          // Re-exports. Nothing to do.
          return;
        }
        node.declaration.as_ref().map(|declaration| self.exec_declaration(declaration, true));
        for specifier in &node.specifiers {
          match &specifier.local {
            ModuleExportName::IdentifierReference(node) => {
              self.exec_identifier_reference_export(node);
            }
            _ => unreachable!(),
          }
        }
      }
      ModuleDeclaration::ExportDefaultDeclaration(node) => {
        let value = match &node.declaration {
          ExportDefaultDeclarationKind::FunctionDeclaration(node) => {
            // Pass `exporting` as `false` because it is actually used as an expression
            self.exec_function(node, false)
          }
          ExportDefaultDeclarationKind::ClassDeclaration(node) => {
            // Pass `exporting` as `false` because it is actually used as an expression
            self.exec_class(node, false)
          }
          node => self.exec_expression(node.to_expression()),
        };
        // FIXME: delay this
        value.consume_as_unknown(self);
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
