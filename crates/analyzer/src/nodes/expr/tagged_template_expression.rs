use crate::{host::Host, 
  analyzer::Analyzer,   dep::DepId,
  };
use oxc::{
  ast::{
    ast::{Expression, TaggedTemplateExpression, TemplateLiteral},
    NONE,
  },
  span::GetSpan,
};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_tagged_template_expression(
    &mut self,
    node: &'a TaggedTemplateExpression<'a>,
  ) -> H::Entity {
    let (_, tag, _, this) = match self.exec_callee(&node.tag) {
      Ok(v) => v,
      Err(v) => return v,
    };

    let mut arguments = vec![(false, self.factory.unknown())];

    for expr in &node.quasi.expressions {
      let value = self.exec_expression(expr);
      let dep = DepId::from(AstKind2::ExpressionInTaggedTemplate(expr));
      arguments.push((false, self.factory.computed(value, dep)));
    }

    let value = tag.call(
      self,
      self.consumable(AstKind2::TaggedTemplateExpression(node)),
      this,
      self.factory.arguments(arguments),
    );

    value
  }
}

