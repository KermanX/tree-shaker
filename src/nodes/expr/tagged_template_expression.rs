use crate::{
  analyzer::Analyzer,
  ast::AstType2,
  build_effect_from_arr,
  entity::{entity::Entity, unknown::UnknownEntity},
  transformer::Transformer,
};
use oxc::ast::ast::{Expression, TSTypeParameterInstantiation, TaggedTemplateExpression};

const AST_TYPE: AstType2 = AstType2::TaggedTemplateExpression;

#[derive(Debug, Default)]
pub struct Data {
  has_effect: bool,
}

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_tagged_template_expression(
    &mut self,
    node: &'a TaggedTemplateExpression<'a>,
  ) -> Entity<'a> {
    let tag = self.exec_expression(&node.tag);
    for expr in &node.quasi.expressions {
      self.exec_expression(expr);
    }

    // TODO: this
    // TODO: more specific arguments
    let (has_effect, ret_val) =
      tag.call(self, &UnknownEntity::new_unknown(), &UnknownEntity::new_unknown());

    let data = self.load_data::<Data>(AST_TYPE, node);
    data.has_effect |= has_effect;

    ret_val
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_tagged_template_expression(
    &mut self,
    node: TaggedTemplateExpression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    let data = self.get_data::<Data>(AST_TYPE, &node);

    let TaggedTemplateExpression { span, tag, quasi, .. } = node;

    let need_call = need_val || data.has_effect;

    if need_call {
      let tag = self.transform_expression(tag, true).unwrap();

      Some(self.ast_builder.expression_tagged_template(
        span,
        tag,
        quasi,
        None::<TSTypeParameterInstantiation>,
      ))
    } else {
      build_effect_from_arr!(
        &self.ast_builder,
        span,
        vec![self.transform_expression(tag, false)],
        quasi.expressions.into_iter().map(|x| self.transform_expression(x, false))
      )
    }
  }
}
