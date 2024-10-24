use super::{
  consumed_object,
  entity::{EnumeratedProperties, IteratedElements},
  Entity, EntityFactory, EntityTrait, TypeofResult,
};
use crate::{
  analyzer::Analyzer,
  ast::AstKind2,
  consumable::{box_consumable, Consumable},
  dep::DepId,
  use_consumed_flag,
};
use oxc::{
  ast::ast::{ArrowFunctionExpression, Class, Function},
  semantic::ScopeId,
  span::{GetSpan, Span},
};
use std::{
  cell::Cell,
  hash::{Hash, Hasher},
  rc::Rc,
};

#[derive(Debug, Clone, Copy)]
pub enum FunctionEntitySource<'a> {
  Function(&'a Function<'a>),
  ArrowFunctionExpression(&'a ArrowFunctionExpression<'a>),
  ClassStatics(&'a Class<'a>),
  ClassConstructor(&'a Class<'a>),
  Module,
}

impl GetSpan for FunctionEntitySource<'_> {
  fn span(&self) -> Span {
    match self {
      FunctionEntitySource::Function(node) => node.span(),
      FunctionEntitySource::ArrowFunctionExpression(node) => node.span(),
      FunctionEntitySource::ClassStatics(node) => node.span(),
      FunctionEntitySource::ClassConstructor(node) => node.span(),
      FunctionEntitySource::Module => Span::default(),
    }
  }
}

impl<'a> FunctionEntitySource<'a> {
  pub fn into_dep_id(self) -> DepId {
    match self {
      FunctionEntitySource::Function(node) => AstKind2::Function(node),
      FunctionEntitySource::ArrowFunctionExpression(node) => {
        AstKind2::ArrowFunctionExpression(node)
      }
      FunctionEntitySource::ClassStatics(node) => AstKind2::Class(node),
      FunctionEntitySource::ClassConstructor(node) => AstKind2::Class(node),
      FunctionEntitySource::Module => AstKind2::Environment,
    }
    .into()
  }

  pub fn name(&self) -> String {
    match self {
      FunctionEntitySource::Function(node) => {
        node.id.as_ref().map_or("<unknown>", |id| &id.name).to_string()
      }
      FunctionEntitySource::ArrowFunctionExpression(_) => "<anonymous>".to_string(),
      FunctionEntitySource::ClassStatics(_) => "<ClassStatics>".to_string(),
      FunctionEntitySource::ClassConstructor(_) => "<ClassConstructor>".to_string(),
      FunctionEntitySource::Module => "<Module>".to_string(),
    }
  }
}

impl PartialEq for FunctionEntitySource<'_> {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (FunctionEntitySource::Module, FunctionEntitySource::Module) => true,
      (FunctionEntitySource::Function(a), FunctionEntitySource::Function(b)) => {
        a.span() == b.span()
      }
      (
        FunctionEntitySource::ArrowFunctionExpression(a),
        FunctionEntitySource::ArrowFunctionExpression(b),
      ) => a.span() == b.span(),
      (FunctionEntitySource::ClassStatics(a), FunctionEntitySource::ClassStatics(b)) => {
        a.span() == b.span()
      }
      _ => false,
    }
  }
}

impl Eq for FunctionEntitySource<'_> {}

impl Hash for FunctionEntitySource<'_> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.span().hash(state)
  }
}

#[derive(Debug, Clone)]
pub struct FunctionEntity<'a> {
  consumed: Rc<Cell<bool>>,
  body_consumed: Rc<Cell<bool>>,
  pub source: FunctionEntitySource<'a>,
  pub variable_scope_stack: Rc<Vec<ScopeId>>,
}

impl<'a> EntityTrait<'a> for FunctionEntity<'a> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    use_consumed_flag!(self);

    self.call_in_recursion(analyzer);
  }

  fn get_property(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
  ) -> Entity<'a> {
    if self.consumed.get() {
      return consumed_object::get_property(rc, analyzer, dep, key);
    }
    analyzer.builtins.prototypes.function.get_property(analyzer, rc, key, dep)
  }

  fn set_property(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
    value: Entity<'a>,
  ) {
    self.consume(analyzer);
    consumed_object::set_property(analyzer, dep, key, value)
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, key: Entity<'a>) {
    self.consume(analyzer);
    consumed_object::delete_property(analyzer, dep, key)
  }

  fn enumerate_properties(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> EnumeratedProperties<'a> {
    if analyzer.config.unknown_property_read_side_effects {
      self.consume(analyzer);
    }
    consumed_object::enumerate_properties(rc, analyzer, dep)
  }

  fn call(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: Entity<'a>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    if self.consumed.get() {
      return consumed_object::call(rc, analyzer, dep, this, args);
    }

    let recursed = analyzer.scope_context.call.iter().any(|scope| scope.source == self.source);
    if recursed {
      self.call_in_recursion(analyzer);
      return consumed_object::call(rc, analyzer, dep, this, args);
    }

    self.call_impl(rc, analyzer, dep, this, args, false)
  }

  fn construct(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    consumed_object::construct(rc, analyzer, dep, args)
  }

  fn r#await(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> Entity<'a> {
    if self.consumed.get() {
      return consumed_object::r#await(analyzer, dep);
    }
    analyzer.factory.computed(rc, dep)
  }

  fn iterate(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> IteratedElements<'a> {
    self.consume(analyzer);
    consumed_object::iterate(analyzer, dep)
  }

  fn get_destructable(&self, rc: Entity<'a>, dep: Consumable<'a>) -> Consumable<'a> {
    box_consumable((rc, dep))
  }

  fn get_typeof(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    analyzer.factory.string("function")
  }

  fn get_to_string(&self, rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    if self.consumed.get() {
      return consumed_object::get_to_string(analyzer);
    }
    analyzer.factory.computed_unknown_string(rc)
  }

  fn get_to_numeric(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    if self.consumed.get() {
      return consumed_object::get_to_numeric(analyzer);
    }
    analyzer.factory.nan
  }

  fn get_to_boolean(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    analyzer.factory.boolean(true)
  }

  fn get_to_property_key(&self, rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    self.get_to_string(rc, analyzer)
  }

  fn test_typeof(&self) -> TypeofResult {
    TypeofResult::Function
  }

  fn test_truthy(&self) -> Option<bool> {
    Some(true)
  }

  fn test_nullish(&self) -> Option<bool> {
    Some(false)
  }
}

impl<'a> FunctionEntity<'a> {
  pub fn call_impl(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: Entity<'a>,
    args: Entity<'a>,
    consume: bool,
  ) -> Entity<'a> {
    if let Some(logger) = analyzer.logger {
      logger.push_fn_call(self.source.span(), self.source.name());
    }

    let call_dep = box_consumable((self.source.into_dep_id(), dep));
    let variable_scopes = self.variable_scope_stack.clone();
    let ret_val = match self.source {
      FunctionEntitySource::Function(node) => analyzer.call_function(
        rc,
        self.source,
        call_dep.cloned(),
        node,
        variable_scopes,
        this.clone(),
        args.clone(),
        consume,
      ),
      FunctionEntitySource::ArrowFunctionExpression(node) => analyzer
        .call_arrow_function_expression(
          self.source,
          call_dep.cloned(),
          node,
          variable_scopes,
          args.clone(),
          consume,
        ),
      _ => unreachable!(),
    };
    analyzer.factory.computed(ret_val, call_dep)
  }

  pub fn call_in_recursion(&self, analyzer: &mut Analyzer<'a>) {
    if self.body_consumed.get() {
      return;
    }
    self.body_consumed.set(true);

    // FIXME: This is not guaranteed to be correct
    // Handle case that a closure is created recursively
    let mut recursion = 0;
    for scope in analyzer.scope_context.call.iter().rev() {
      if scope.source == self.source {
        recursion += 1;
        if recursion > 1 {
          return;
        }
      }
    }

    analyzer.consume(self.source.into_dep_id());
    analyzer.consume_arguments(Some(self.source));

    let self_cloned = self.clone();
    analyzer.exec_consumed_fn(move |analyzer| {
      self_cloned.call_impl(
        analyzer.factory.unknown(),
        analyzer,
        box_consumable(()),
        analyzer.factory.unknown(),
        analyzer.factory.unknown(),
        true,
      )
    });
  }
}

impl<'a> EntityFactory<'a> {
  pub fn function(
    &self,
    source: FunctionEntitySource<'a>,
    variable_scope_stack: Vec<ScopeId>,
  ) -> Entity<'a> {
    self.entity(FunctionEntity {
      consumed: Rc::new(Cell::new(false)),
      body_consumed: Rc::new(Cell::new(false)),
      source,
      variable_scope_stack: Rc::new(variable_scope_stack),
    })
  }
}
