use crate::{analyzer::Analyzer, host::Host};
use oxc::{
  allocator,
  ast::ast::{Expression, JSXMemberExpression},
};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_jsx_member_expression(&mut self, node: &'a JSXMemberExpression<'a>) -> H::Entity {
    let object = self.exec_jsx_member_expression_object(&node.object);
    let key = self.factory.string(&node.property.name);
    object.get_property(self, self.consumable(AstKind2::JSXMemberExpression(node)), key)
  }
}
