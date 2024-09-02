use crate::{
  analyzer::Analyzer,
  ast::AstType2,
  entity::{entity::Entity, unknown::UnknownEntity},
  transformer::Transformer,
};
use oxc::ast::ast::{Expression, MetaProperty};

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_meta_property(&mut self, node: &'a MetaProperty<'a>) -> Entity<'a> {
    let meta = node.meta.name.as_str();
    let property = node.property.name.as_str();

    if meta == "import" && property == "meta" {
      self.builtins.get_import_meta()
    } else {
      UnknownEntity::new_unknown()
    }
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_meta_property(
    &mut self,
    node: MetaProperty<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    need_val.then(|| self.ast_builder.expression_from_meta_property(node))
  }
}
