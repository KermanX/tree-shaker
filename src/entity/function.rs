use super::{
  consumed_object, ComputedEntity, Consumable, Entity, EntityDepNode, EntityTrait, ForwardedEntity,
  LiteralEntity, TypeofResult, UnknownEntity,
};
use crate::{analyzer::Analyzer, use_consumed_flag};
use oxc::{
  ast::{
    ast::{ArrowFunctionExpression, Function},
    AstKind,
  },
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
  Module,
}

impl GetSpan for FunctionEntitySource<'_> {
  fn span(&self) -> Span {
    match self {
      FunctionEntitySource::Function(node) => node.span(),
      FunctionEntitySource::ArrowFunctionExpression(node) => node.span(),
      FunctionEntitySource::Module => Span::default(),
    }
  }
}

impl<'a> FunctionEntitySource<'a> {
  pub fn into_dep_node(self) -> EntityDepNode {
    match self {
      FunctionEntitySource::Function(node) => AstKind::Function(node).into(),
      FunctionEntitySource::ArrowFunctionExpression(node) => {
        AstKind::ArrowFunctionExpression(node).into()
      }
      FunctionEntitySource::Module => EntityDepNode::Environment,
    }
  }

  pub fn name(&self) -> String {
    match self {
      FunctionEntitySource::Function(node) => {
        node.id.as_ref().map_or("<unknown>", |id| &id.name).to_string()
      }
      FunctionEntitySource::ArrowFunctionExpression(_) => "<anonymous>".to_string(),
      FunctionEntitySource::Module => "<Module>".to_string(),
    }
  }
}

impl PartialEq for FunctionEntitySource<'_> {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (FunctionEntitySource::Function(a), FunctionEntitySource::Function(b)) => {
        a.span() == b.span()
      }
      (
        FunctionEntitySource::ArrowFunctionExpression(a),
        FunctionEntitySource::ArrowFunctionExpression(b),
      ) => a.span() == b.span(),
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
  pub is_expression: bool,
}

impl<'a> EntityTrait<'a> for FunctionEntity<'a> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    use_consumed_flag!(self);

    self.call_in_recursion(analyzer);
  }

  fn get_property(
    &self,
    rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: &Entity<'a>,
  ) -> Entity<'a> {
    if self.consumed.get() {
      return consumed_object::get_property(rc, analyzer, dep, key);
    }
    analyzer.builtins.prototypes.function.get_property(rc, key, dep)
  }

  fn set_property(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: &Entity<'a>,
    value: Entity<'a>,
  ) {
    self.consume(analyzer);
    consumed_object::set_property(analyzer, dep, key, value)
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, key: &Entity<'a>) {
    self.consume(analyzer);
    consumed_object::delete_property(analyzer, dep, key)
  }

  fn enumerate_properties(
    &self,
    rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> Vec<(bool, Entity<'a>, Entity<'a>)> {
    if analyzer.config.unknown_property_read_side_effects {
      self.consume(analyzer);
    }
    consumed_object::enumerate_properties(rc, analyzer, dep)
  }

  fn call(
    &self,
    rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: &Entity<'a>,
    args: &Entity<'a>,
  ) -> Entity<'a> {
    if self.consumed.get() {
      return consumed_object::call(analyzer, dep, this, args);
    }

    let recursed = analyzer.scope_context.call.iter().any(|scope| scope.source == self.source);
    if recursed {
      self.call_in_recursion(analyzer);
      return consumed_object::call(analyzer, dep, this, args);
    }

    self.call_impl(rc, analyzer, dep, this, args, false)
  }

  fn r#await(
    &self,
    rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> Entity<'a> {
    if self.consumed.get() {
      return consumed_object::r#await(analyzer, dep);
    }
    ComputedEntity::new(rc.clone(), dep)
  }

  fn iterate(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> (Vec<Entity<'a>>, Option<Entity<'a>>) {
    self.consume(analyzer);
    consumed_object::iterate(analyzer, dep)
  }

  fn get_typeof(&self) -> Entity<'a> {
    LiteralEntity::new_string("function")
  }

  fn get_to_string(&self, rc: &Entity<'a>) -> Entity<'a> {
    if self.consumed.get() {
      return consumed_object::get_to_string();
    }
    UnknownEntity::new_computed_string(rc.clone())
  }

  fn get_to_numeric(&self, _rc: &Entity<'a>) -> Entity<'a> {
    if self.consumed.get() {
      return consumed_object::get_to_numeric();
    }
    LiteralEntity::new_nan()
  }

  fn get_to_boolean(&self, _rc: &Entity<'a>) -> Entity<'a> {
    LiteralEntity::new_boolean(true)
  }

  fn get_to_property_key(&self, rc: &Entity<'a>) -> Entity<'a> {
    self.get_to_string(rc)
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
  pub fn new(
    source: FunctionEntitySource<'a>,
    variable_scope_stack: Vec<ScopeId>,
    is_expression: bool,
  ) -> Entity<'a> {
    Entity::new(Self {
      consumed: Rc::new(Cell::new(false)),
      body_consumed: Rc::new(Cell::new(false)),
      source,
      variable_scope_stack: Rc::new(variable_scope_stack),
      is_expression,
    })
  }

  pub fn call_impl(
    &self,
    rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: &Entity<'a>,
    args: &Entity<'a>,
    consume_return: bool,
  ) -> Entity<'a> {
    if let Some(logger) = analyzer.logger {
      logger.push_fn_call(self.source.span(), self.source.name());
    }

    let call_dep: Consumable<'a> = (self.source.into_dep_node(), dep).into();
    let variable_scopes = self.variable_scope_stack.clone();
    let ret_val = match self.source {
      FunctionEntitySource::Function(node) => analyzer.call_function(
        rc.clone(),
        self.source,
        self.is_expression,
        call_dep.clone(),
        node,
        variable_scopes,
        this.clone(),
        args.clone(),
        consume_return,
      ),
      FunctionEntitySource::ArrowFunctionExpression(node) => analyzer
        .call_arrow_function_expression(
          self.source,
          call_dep.clone(),
          node,
          variable_scopes,
          args.clone(),
          consume_return,
        ),
      FunctionEntitySource::Module => unreachable!(),
    };
    ForwardedEntity::new(ret_val, call_dep)
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

    analyzer.consume(self.source.into_dep_node());
    analyzer.consume_arguments(Some(self.source));

    let self_cloned = self.clone();
    analyzer.exec_consumed_fn(move |analyzer| {
      self_cloned.call_impl(
        &UnknownEntity::new_unknown(),
        analyzer,
        ().into(),
        &UnknownEntity::new_unknown(),
        &UnknownEntity::new_unknown(),
        true,
      )
    });
  }
}
