use crate::{transformer::Transformer, Analyzer};
use oxc::ast::ast::{
  ExportDefaultDeclaration, ExportDefaultDeclarationKind, ExportNamedDeclaration,
  ModuleDeclaration, ModuleExportName,
};

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_module_declaration(&mut self, node: &'a ModuleDeclaration<'a>) {
    match node {
      ModuleDeclaration::ExportNamedDeclaration(node) => {
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
          ExportDefaultDeclarationKind::ClassDeclaration(node) => todo!(),
          node => self.exec_expression(node.to_expression()),
        };
        // FIXME: delay this
        value.consume_as_unknown(self);
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
      ModuleDeclaration::ExportDefaultDeclaration(node) => {
        let ExportDefaultDeclaration { span, declaration, exported, .. } = node.unbox();
        let declaration = match declaration {
          ExportDefaultDeclarationKind::FunctionDeclaration(node) => {
            let function = self.transform_function(node.unbox(), true).unwrap();
            self.ast_builder.export_default_declaration_kind_from_function(function)
          }
          ExportDefaultDeclarationKind::ClassDeclaration(node) => todo!(),
          node => {
            let expression = self.transform_expression(node.try_into().unwrap(), true).unwrap();
            self.ast_builder.export_default_declaration_kind_expression(expression)
          }
        };
        self.ast_builder.module_declaration_export_default_declaration(span, declaration, exported)
      }
      _ => todo!(),
    }
  }
}
