use crate::{host::Host, analyzer::Analyzer};
use oxc::ast::ast::{Expression, MetaProperty};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_meta_property(&mut self, node: &'a MetaProperty<'a>) -> H::Entity {
    let meta = node.meta.name.as_str();
    let property = node.property.name.as_str();

    if meta == "import" && property == "meta" {
      self.builtins.import_meta
    } else {
      self.factory.unknown()
    }
  }
}
