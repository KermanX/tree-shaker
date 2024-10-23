use crate::{analyzer::Analyzer, entity::Entity, transformer::Transformer};
use oxc::ast::ast::{Expression, Super};

impl<'a> Analyzer<'a> {
  pub fn exec_super(&mut self, _node: &'a Super) -> Entity<'a> {
    self.get_super()
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_super(&self, node: &'a Super, need_val: bool) -> Option<Expression<'a>> {
    if need_val {
      Some(self.ast_builder.expression_super(node.span))
    } else {
      None
    }
  }
}
