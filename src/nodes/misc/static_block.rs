use crate::{
  analyzer::Analyzer,
  ast::AstType2,
  data::StatementVecData,
  entity::{dep::EntityDepNode, unknown::UnknownEntity},
  transformer::Transformer,
};
use oxc::ast::{ast::{ClassElement, StaticBlock}, AstKind};
use std::rc::Rc;

const AST_TYPE: AstType2 = AstType2::StaticBlock;

impl<'a> Analyzer<'a> {
  pub fn exec_static_block(&mut self, node: &'a StaticBlock<'a>) {
    self.push_call_scope(
      AstKind::StaticBlock(node),
      EntityDepNode::Environment,
      Rc::new(self.scope_context.variable_scopes.clone()),
      UnknownEntity::new_unknown(),
      false,
      false,
    );

    let data = self.load_data::<StatementVecData>(AST_TYPE, node);
    self.exec_statement_vec(data, &node.body);

    self.pop_call_scope();
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_static_block(&self, node: &'a StaticBlock<'a>) -> ClassElement<'a> {
    let data = self.get_data::<StatementVecData>(AST_TYPE, node);

    let StaticBlock { span, body, .. } = node;

    let body = self.transform_statement_vec(data, body);

    self.ast_builder.class_element_static_block(*span, body)
  }
}
