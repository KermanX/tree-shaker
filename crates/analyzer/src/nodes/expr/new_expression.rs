use crate::{host::Host, 
  analyzer::Analyzer,  
};
use oxc::ast::ast::{Expression, NewExpression, TSTypeParameterInstantiation};

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn exec_new_expression(&mut self, node: &'a NewExpression<'a>) -> H::Entity {
    let pure = self.has_pure_notation(node.span);

    self.scope_context.pure += pure;
    let callee = self.exec_expression(&node.callee);
    self.scope_context.pure -= pure;

    let arguments = self.exec_arguments(&node.arguments);

    self.scope_context.pure += pure;
    let value = callee.construct(self, self.consumable(AstKind2::NewExpression(node)), arguments);
    self.scope_context.pure -= pure;

    value
  }
}

