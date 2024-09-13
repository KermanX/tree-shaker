use crate::ast::DeclarationKind;
use crate::entity::dep::EntityDepNode;
use crate::entity::entity::Entity;
use crate::entity::function::FunctionEntity;
use crate::{transformer::Transformer, Analyzer};
use oxc::ast::ast::{Function, TSThisParameter, TSTypeAnnotation, TSTypeParameterDeclaration};

impl<'a> Analyzer<'a> {
  pub fn exec_function(&mut self, node: &'a Function<'a>) -> Entity<'a> {
    let dep = self.new_entity_dep(EntityDepNode::Function(node));
    FunctionEntity::new(dep.clone())
  }

  pub fn declare_function(&mut self, node: &'a Function<'a>, exporting: bool) {
    let dep = self.new_entity_dep(EntityDepNode::Function(node));
    let entity = self.exec_function(node);

    let symbol = node.id.as_ref().unwrap().symbol_id.get().unwrap();
    self.declare_symbol(symbol, dep, exporting, DeclarationKind::Function, Some(entity.clone()));
  }

  pub fn call_function(
    &mut self,
    node: &'a Function<'a>,
    this: Entity<'a>,
    args: Entity<'a>,
  ) -> (bool, Entity<'a>) {
    self.push_function_scope(this, node.r#async, node.generator);

    self.exec_formal_parameters(&node.params, args);
    self.exec_function_body(node.body.as_ref().unwrap());

    self.pop_function_scope()
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_function(&self, node: &'a Function<'a>, need_val: bool) -> Option<Function<'a>> {
    if need_val || self.is_referred(EntityDepNode::Function(&node)) {
      let Function { r#type, span, id, generator, r#async, params, body, .. } = node;

      let params = self.transform_formal_parameters(params);
      let body = self.transform_function_body(body.as_ref().unwrap());

      Some(self.ast_builder.function(
        *r#type,
        *span,
        id.clone(),
        *generator,
        *r#async,
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
