use crate::entity::dep::EntityDep;
use crate::entity::entity::Entity;
use crate::entity::function::FunctionEntity;
use crate::{transformer::Transformer, Analyzer};
use oxc::ast::ast::Function;

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_function(&mut self, node: &'a Function<'a>) -> Entity<'a> {
    let entity: Entity<'a> = FunctionEntity::new(EntityDep::Function(node));

    if let Some(id) = &node.id {
      let symbol = id.symbol_id.get().unwrap();
      self.variable_scope_mut().declare(symbol, entity.clone());
    }

    entity
  }

  pub(crate) fn call_function(
    &mut self,
    node: &'a Function<'a>,
    this: Entity<'a>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    self.push_variable_scope();
    self.push_function_scope();

    self.exec_formal_parameters(&node.params, args);

    if let Some(body) = &node.body {
      for statement in &body.statements {
        self.exec_statement(statement);
      }
    }

    self.pop_variable_scope();
    self.pop_function_scope().returned_value()
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_function(&self, node: Function<'a>, need_val: bool) -> Option<Function<'a>> {
    if need_val || self.is_referred(EntityDep::Function(&node)) {
      Some(node)
    } else {
      None
    }
  }
}
