use crate::{
  analyzer::Analyzer,
  ast::DeclarationKind,
  entity::{
    consumable::Consumable,
    dep::EntityDepNode,
    entity::Entity,
    function::{FunctionEntity, FunctionEntitySource},
    unknown::UnknownEntity,
  },
  scope::variable_scope::VariableScopes,
  transformer::Transformer,
};
use oxc::ast::{
  ast::{Function, TSThisParameter, TSTypeAnnotation, TSTypeParameterDeclaration},
  AstKind,
};
use std::rc::Rc;

impl<'a> Analyzer<'a> {
  pub fn exec_function(&mut self, node: &'a Function<'a>, is_expression: bool) -> Entity<'a> {
    FunctionEntity::new(
      FunctionEntitySource::Function(node),
      self.scope_context.variable_scopes.clone(),
      is_expression,
    )
  }

  pub fn declare_function(&mut self, node: &'a Function<'a>, exporting: bool) {
    let dep = AstKind::Function(node);
    let entity = self.exec_function(node, false);

    let symbol = node.id.as_ref().unwrap().symbol_id.get().unwrap();
    self.declare_symbol(symbol, dep, exporting, DeclarationKind::Function, Some(entity.clone()));
  }

  pub fn call_function(
    &mut self,
    fn_entity: Entity<'a>,
    decl_dep: Consumable<'a>,
    source: EntityDepNode,
    is_expression: bool,
    call_dep: Consumable<'a>,
    node: &'a Function<'a>,
    variable_scopes: Rc<VariableScopes<'a>>,
    this: Entity<'a>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    let runner: Box<dyn Fn(&mut Analyzer<'a>) -> Entity<'a> + 'a> =
      Box::new(move |analyzer: &mut Analyzer<'a>| {
        analyzer.push_call_scope(
          source,
          call_dep.clone(),
          variable_scopes.clone(),
          this.clone(),
          (args.clone(), vec![ /* later filled by formal parameters */]),
          node.r#async,
          node.generator,
        );

        let declare_in_body = is_expression && node.id.is_some();
        if declare_in_body {
          let symbol = node.id.as_ref().unwrap().symbol_id.get().unwrap();
          analyzer.declare_symbol(
            symbol,
            decl_dep.clone(),
            false,
            DeclarationKind::NamedFunctionInBody,
            Some(fn_entity.clone()),
          );

          analyzer.push_variable_scope();
          analyzer.call_scope_mut().variable_scope_index += 1;
        }

        analyzer.exec_formal_parameters(
          &node.params,
          args.clone(),
          DeclarationKind::FunctionParameter,
        );
        analyzer.exec_function_body(node.body.as_ref().unwrap());

        if declare_in_body {
          analyzer.pop_variable_scope();
        }

        analyzer.pop_call_scope()
      });

    if node.generator {
      // Too complex to analyze the control flow, thus run exhaustively
      self.exec_exhaustively(move |analyzer| {
        runner(analyzer).consume(analyzer);
      });
      UnknownEntity::new_unknown()
    } else {
      runner(self)
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_function(&self, node: &'a Function<'a>, need_val: bool) -> Option<Function<'a>> {
    if need_val || self.is_referred(AstKind::Function(&node)) {
      let Function { r#type, span, id, generator, r#async, params, body, .. } = node;

      self.call_stack.borrow_mut().push(AstKind::Function(node).into());

      let params = self.transform_formal_parameters(params);
      let body = self.transform_function_body(body.as_ref().unwrap());

      self.call_stack.borrow_mut().pop();

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
