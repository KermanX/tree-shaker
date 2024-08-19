use crate::TreeShaker;
use oxc::ast::ast::ExportNamedDeclaration;

#[derive(Debug, Default, Clone)]
pub struct Data {
  included: bool,
}

impl<'a> TreeShaker<'a> {
  pub(crate) fn exec_export_named_declaration(&mut self, node: &'a ExportNamedDeclaration) {
    let data = self.load_data::<Data>(node);
    data.included = true;
    node.declaration.as_ref().map(|declaration| self.exec_declaration(declaration, None));
  }
}
