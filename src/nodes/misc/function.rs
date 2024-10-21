use crate::{
  analyzer::Analyzer,
  ast::DeclarationKind,
  consumable::{box_consumable, Consumable},
  entity::{Entity, FunctionEntitySource},
  transformer::Transformer,
};
use oxc::{
  ast::{
    ast::{Function, TSThisParameter, TSTypeAnnotation, TSTypeParameterDeclaration},
    AstKind,
  },
  semantic::ScopeId,
};
use std::rc::Rc;

impl<'a> Analyzer<'a> {
  pub fn exec_function(&mut self, node: &'a Function<'a>, is_expression: bool) -> Entity<'a> {
    self.factory.function(
      FunctionEntitySource::Function(node),
      self.scope_context.variable.stack.clone(),
      is_expression,
    )
  }

  pub fn declare_function(&mut self, node: &'a Function<'a>, exporting: bool) {
    let dep = box_consumable(AstKind::Function(node));
    let entity = self.exec_function(node, false);

    let symbol = node.id.as_ref().unwrap().symbol_id.get().unwrap();
    self.declare_symbol(symbol, dep, exporting, DeclarationKind::Function, Some(entity.clone()));
  }

  pub fn call_function(
    &mut self,
    fn_entity: Entity<'a>,
    source: FunctionEntitySource<'a>,
    is_expression: bool,
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
          source,
          call_dep.cloned(),
          variable_scopes.as_ref().clone(),
          this.clone(),
          (args.clone(), vec![ /* later filled by formal parameters */]),
          node.r#async,
          node.generator,
          consume,
        );

        let declare_in_body = is_expression && node.id.is_some();
        if declare_in_body {
          let symbol = node.id.as_ref().unwrap().symbol_id.get().unwrap();
          analyzer.declare_symbol(
            symbol,
            box_consumable(source.into_dep_id()),
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
      self.factory.unknown
    } else {
      runner(self)
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_function(&self, node: &'a Function<'a>, need_val: bool) -> Option<Function<'a>> {
    if need_val || self.is_referred(AstKind::Function(&node)) {
      let Function { r#type, span, id, generator, r#async, params, body, .. } = node;

      self.call_stack.borrow_mut().push(FunctionEntitySource::Function(node));

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
