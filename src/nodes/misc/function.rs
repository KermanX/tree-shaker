use crate::{
  analyzer::Analyzer,
  ast::{AstKind2, DeclarationKind},
  consumable::{box_consumable, Consumable},
  entity::Entity,
  transformer::Transformer,
  utils::{CalleeInfo, CalleeNode},
};
use oxc::{
  allocator,
  ast::ast::{
    Function, FunctionType, TSThisParameter, TSTypeAnnotation, TSTypeParameterDeclaration,
  },
  semantic::ScopeId,
};
use std::rc::Rc;

impl<'a> Analyzer<'a> {
  pub fn exec_function(&mut self, node: &'a Function<'a>) -> Entity<'a> {
    self.new_function(CalleeNode::Function(node))
  }

  pub fn declare_function(&mut self, node: &'a Function<'a>, exporting: bool) {
    let dep = box_consumable(AstKind2::Function(node));
    let entity = self.exec_function(node);

    let symbol = node.id.as_ref().unwrap().symbol_id.get().unwrap();
    self.declare_symbol(symbol, dep, exporting, DeclarationKind::Function, Some(entity.clone()));
  }

  pub fn call_function(
    &mut self,
    fn_entity: Entity<'a>,
    callee: CalleeInfo<'a>,
    call_dep: Consumable<'a>,
    node: &'a Function<'a>,
    variable_scopes: Rc<Vec<ScopeId>>,
    this: Entity<'a>,
    args: Entity<'a>,
    consume: bool,
  ) -> Entity<'a> {
    let runner: Box<dyn Fn(&mut Analyzer<'a>) -> Entity<'a> + 'a> =
      Box::new(move |analyzer: &mut Analyzer<'a>| {
        analyzer.push_call_scope(
          callee,
          call_dep.cloned(),
          variable_scopes.as_ref().clone(),
          node.r#async,
          node.generator,
          consume,
        );

        let variable_scope = analyzer.variable_scope_mut();
        variable_scope.this = Some(this);
        variable_scope.arguments =
          Some((args.clone(), vec![ /* later filled by formal parameters */]));

        let declare_in_body = node.r#type == FunctionType::FunctionExpression && node.id.is_some();
        if declare_in_body {
          let symbol = node.id.as_ref().unwrap().symbol_id.get().unwrap();
          analyzer.declare_symbol(
            symbol,
            box_consumable(callee.into_dep_id()),
            false,
            DeclarationKind::NamedFunctionInBody,
            Some(fn_entity.clone()),
          );
        }

        analyzer.exec_formal_parameters(
          &node.params,
          args.clone(),
          DeclarationKind::FunctionParameter,
        );
        analyzer.exec_function_body(node.body.as_ref().unwrap());

        if consume {
          analyzer.consume_return_values();
        }

        analyzer.pop_call_scope()
      });

    if node.r#async || node.generator {
      // Too complex to analyze the control flow, thus run exhaustively
      self.exec_async_or_generator_fn(move |analyzer| {
        runner(analyzer).consume(analyzer);
      });
      self.factory.unknown()
    } else {
      runner(self)
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_function(
    &self,
    node: &'a Function<'a>,
    need_val: bool,
  ) -> Option<allocator::Box<'a, Function<'a>>> {
    if need_val || self.is_referred(AstKind2::Function(&node)) {
      let Function { r#type, span, id, generator, r#async, params, body, .. } = node;

      let old_declaration_only = self.declaration_only.replace(false);

      let params = self.transform_formal_parameters(params);

      let body =
        body.as_ref().map(|body| self.transform_function_body(node.scope_id.get().unwrap(), body));

      if let Some(id) = id {
        let symbol = id.symbol_id.get().unwrap();
        self.update_var_decl_state(symbol, true);
      }

      self.declaration_only.set(old_declaration_only);

      Some(self.ast_builder.alloc_function(
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
        body,
      ))
    } else {
      None
    }
  }
}
