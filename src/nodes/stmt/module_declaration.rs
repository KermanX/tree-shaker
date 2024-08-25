use crate::{transformer::Transformer, Analyzer};
use oxc::ast::ast::{ExportNamedDeclaration, ModuleDeclaration, ModuleExportName};

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_module_declaration(&mut self, node: &'a ModuleDeclaration<'a>) {
    match node {
      ModuleDeclaration::ExportNamedDeclaration(node) => {
        self.exporting = true;
        node.declaration.as_ref().map(|declaration| self.exec_declaration(declaration));
        for specifier in &node.specifiers {
          match &specifier.local {
            ModuleExportName::IdentifierReference(node) => {
              self.exec_identifier_reference_export(node);
            }
            _ => unreachable!(),
          }
        }
        self.exporting = false;
      }
      ModuleDeclaration::ExportDefaultDeclaration(_node) => {
        todo!()
      }
      _ => todo!(),
    }
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_module_declaration(
    &mut self,
    node: ModuleDeclaration<'a>,
  ) -> ModuleDeclaration<'a> {
    match node {
      ModuleDeclaration::ExportNamedDeclaration(node) => {
        let ExportNamedDeclaration {
          span,
          declaration,
          specifiers,
          source,
          export_kind,
          with_clause,
          ..
        } = node.unbox();
        let declaration = declaration.and_then(|d| self.transform_declaration(d));
        self.ast_builder.module_declaration_export_named_declaration(
          span,
          declaration,
          specifiers,
          source,
          export_kind,
          with_clause,
        )
      }
      _ => todo!(),
    }
  }
}
