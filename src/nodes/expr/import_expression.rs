use crate::{
  analyzer::Analyzer,
  build_effect_from_arr,
  entity::{entity::Entity, unknown::UnknownEntity},
  transformer::Transformer,
};
use oxc::ast::ast::{Expression, ImportExpression};

impl<'a> Analyzer<'a> {
  pub fn exec_import_expression(&mut self, node: &'a ImportExpression<'a>) -> Entity<'a> {
    self.exec_expression(&node.source);

    for argument in &node.arguments {
      self.exec_expression(argument);
    }

    UnknownEntity::new_unknown()
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_import_expression(
    &mut self,
    node: ImportExpression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    let ImportExpression { span, source, arguments, .. } = node;

    let source = self.transform_expression(source, need_val);

    // FIXME: side effects

    if need_val {
      let mut transformed_arguments = self.ast_builder.vec();
      for argument in arguments {
        transformed_arguments.push(self.transform_expression(argument, true).unwrap());
      }
      Some(self.ast_builder.expression_import(span, source.unwrap(), transformed_arguments))
    } else {
      let mut effects = vec![source];
      for argument in arguments {
        effects.push(self.transform_expression(argument, false));
      }
      build_effect_from_arr!(&self.ast_builder, span, effects)
    }
  }
}
