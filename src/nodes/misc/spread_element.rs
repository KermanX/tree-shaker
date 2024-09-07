use crate::{analyzer::Analyzer, ast::AstType2, entity::entity::Entity, transformer::Transformer};
use oxc::ast::ast::{ArrayExpressionElement, SpreadElement};

const AST_TYPE: AstType2 = AstType2::SpreadElement;

#[derive(Debug, Default)]
struct Data {
  has_effect: bool,
}

impl<'a> Analyzer<'a> {
  pub fn exec_spread_element(&mut self, node: &'a SpreadElement<'a>) -> Option<Entity<'a>> {
    let argument = self.exec_expression(&node.argument);
    let (has_effect, iterated) = argument.iterate(self);

    let data = self.load_data::<Data>(AST_TYPE, node);
    data.has_effect |= has_effect;

    iterated
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_spread_element(
    &self,
    node: &'a SpreadElement<'a>,
    need_val: bool,
  ) -> Option<ArrayExpressionElement<'a>> {
    let data = self.get_data::<Data>(AST_TYPE, node);

    let SpreadElement { span, argument } = node;

    let need_spread = need_val || data.has_effect;

    let argument = self.transform_expression(argument, need_spread);

    if let Some(argument) = argument {
      Some(if need_spread {
        self.ast_builder.array_expression_element_spread_element(*span, argument)
      } else {
        self.ast_builder.array_expression_element_expression(argument)
      })
    } else {
      None
    }
  }
}
