use std::mem;

use crate::{
  analyzer::Analyzer,
  ast::AstType2,
  consumable::box_consumable,
  data::StatementVecData,
  entity::{Entity, FunctionEntitySource},
  transformer::Transformer,
};
use oxc::ast::ast::{ClassElement, StaticBlock};

const AST_TYPE: AstType2 = AstType2::StaticBlock;

impl<'a> Analyzer<'a> {
  pub fn exec_static_block(&mut self, node: &'a StaticBlock<'a>, class: Entity<'a>) {
    let data = self.load_data::<StatementVecData>(AST_TYPE, node);

    let variable_scope_stack = mem::take(&mut self.scope_context.variable.stack);
    self.push_call_scope(
      FunctionEntitySource::StaticBlock(node),
      box_consumable(()),
      variable_scope_stack,
      class,
      (self.factory.unknown, vec![]),
      false,
      false,
      false,
    );

    self.exec_statement_vec(data, &node.body);

    self.pop_call_scope();
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_static_block(&self, node: &'a StaticBlock<'a>) -> Option<StaticBlock<'a>> {
    let data = self.get_data::<StatementVecData>(AST_TYPE, node);

    let StaticBlock { span, body, .. } = node;

    let body = self.transform_statement_vec(data, body);

    (!body.is_empty()).then(|| self.ast_builder.static_block(*span, body))
  }
}
