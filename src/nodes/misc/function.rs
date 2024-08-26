use crate::entity::dep::EntityDepNode;
use crate::entity::entity::Entity;
use crate::entity::function::FunctionEntity;
use crate::{transformer::Transformer, Analyzer};
use oxc::ast::ast::{Function, TSThisParameter, TSTypeAnnotation, TSTypeParameterDeclaration};

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_function(&mut self, node: &'a Function<'a>, exporting: bool) -> Entity<'a> {
    let dep = self.new_entity_dep(EntityDepNode::Function(node));
    let entity = FunctionEntity::new(dep);

    if let Some(id) = &node.id {
      let symbol = id.symbol_id.get().unwrap();
      self.declare_symbol(symbol, entity.clone(), exporting);
    }

    entity
  }

  pub(crate) fn call_function(
    &mut self,
    node: &'a Function<'a>,
    this: Entity<'a>,
    args: Entity<'a>,
  ) -> (bool, Entity<'a>) {
    self.push_variable_scope();
    self.push_function_scope();

    self.exec_formal_parameters(&node.params, args);
    self.exec_function_body(node.body.as_ref().unwrap());

    self.pop_variable_scope();
    self.pop_function_scope().get_result()
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_function(&mut self, node: Function<'a>, need_val: bool) -> Option<Function<'a>> {
    if need_val || self.is_referred(EntityDepNode::Function(&node)) {
      let Function { r#type, span, id, generator, r#async, params, body, .. } = node;

      let params = self.transform_formal_parameters(params.unbox());
      let body = self.transform_function_body(body.unwrap().unbox());

      Some(self.ast_builder.function(
        r#type,
        span,
        id,
        generator,
        r#async,
        false,
        None::<TSTypeParameterDeclaration>,
        None::<TSThisParameter>,
        params,
        None::<TSTypeAnnotation>,
        Some(body),
      ))
    } else {
      None
    }
  }
}