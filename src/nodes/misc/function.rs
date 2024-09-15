use crate::{
  analyzer::Analyzer,
  ast::DeclarationKind,
  entity::{
    dep::EntityDep,
    entity::Entity,
    function::{FunctionEntity, FunctionEntitySource},
  },
  scope::variable_scope::VariableScopes,
  transformer::Transformer,
};
use oxc::{ast::{
  ast::{Function, TSThisParameter, TSTypeAnnotation, TSTypeParameterDeclaration},
  AstKind,
}, syntax::symbol};
use std::rc::Rc;

impl<'a> Analyzer<'a> {
  pub fn exec_function(&mut self, node: &'a Function<'a>) -> Entity<'a> {
    FunctionEntity::new(
      FunctionEntitySource::Function(node),
      self.scope_context.variable_scopes.clone(),
    )
  }

  pub fn declare_function(&mut self, node: &'a Function<'a>, exporting: bool) {
    let dep = AstKind::Function(node);
    let entity = self.exec_function(node);

    let symbol = node.id.as_ref().unwrap().symbol_id.get().unwrap();
    self.declare_symbol(symbol, dep, exporting, DeclarationKind::Function, Some(entity.clone()));
  }

  pub fn call_function(
    &mut self,
    fn_entity: Entity<'a>,
    decl_dep: EntityDep,
    call_dep: EntityDep,
    node: &'a Function<'a>,
    variable_scopes: Rc<VariableScopes<'a>>,
    this: Entity<'a>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    self.push_call_scope(call_dep, variable_scopes, this, node.r#async, node.generator);

    if let Some(id) = node.id.as_ref() {
      let symbol = id.symbol_id.get().unwrap();
      self.declare_symbol(symbol, decl_dep, false, DeclarationKind::Function, Some(fn_entity));
    }

    self.exec_formal_parameters(&node.params, args);
    self.exec_function_body(node.body.as_ref().unwrap());

    self.pop_call_scope()
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_function(&self, node: &'a Function<'a>, need_val: bool) -> Option<Function<'a>> {
    if need_val || self.is_referred(AstKind::Function(&node)) {
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
