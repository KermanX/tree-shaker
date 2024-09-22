use super::{
  dep::{EntityDep, EntityDepNode},
  entity::{Entity, EntityTrait},
  forwarded::ForwardedEntity,
  literal::LiteralEntity,
  typeof_result::TypeofResult,
  unknown::{UnknownEntity, UnknownEntityKind},
};
use crate::{
  analyzer::Analyzer,
  entity::consumed_object,
  scope::variable_scope::{VariableScope, VariableScopes},
  use_consumed_flag,
};
use oxc::ast::{
  ast::{ArrowFunctionExpression, Function},
  AstKind,
};
use std::{
  cell::{Cell, RefCell},
  rc::Rc,
};

#[derive(Debug, Clone, Copy)]
pub enum FunctionEntitySource<'a> {
  Function(&'a Function<'a>),
  ArrowFunctionExpression(&'a ArrowFunctionExpression<'a>),
}

#[derive(Debug, Clone)]
pub struct FunctionEntity<'a> {
  consumed: Rc<Cell<bool>>,
  pub source: FunctionEntitySource<'a>,
  pub variable_scopes: Rc<VariableScopes<'a>>,
  pub is_expression: bool,
}

impl<'a> EntityTrait<'a> for FunctionEntity<'a> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    use_consumed_flag!(self);

    analyzer.refer_dep(self.dep());

    let self_cloned = self.clone();
    analyzer.exec_exhaustively(move |analyzer| {
      analyzer.push_cf_scope_normal(None);
      analyzer.push_try_scope();

      let ret_val = self_cloned.call_impl(
        &UnknownEntity::new_unknown(),
        analyzer,
        (EntityDepNode::Environment).into(),
        &UnknownEntity::new_unknown(),
        &UnknownEntity::new_unknown(),
      );
      ret_val.consume(analyzer);

      analyzer.pop_try_scope().thrown_val().map(|thrown_val| {
        thrown_val.consume(analyzer);
      });
      analyzer.pop_cf_scope();
    });
  }

  fn get_property(
    &self,
    rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
    key: &Entity<'a>,
  ) -> Entity<'a> {
    if self.consumed.get() {
      return consumed_object::get_property(analyzer, dep, key);
    }
    analyzer.builtins.prototypes.function.get_property(rc, key, dep)
  }

  fn set_property(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
    key: &Entity<'a>,
    value: Entity<'a>,
  ) {
    self.consume(analyzer);
    consumed_object::set_property(analyzer, dep, key, value)
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, dep: EntityDep, key: &Entity<'a>) {
    self.consume(analyzer);
    consumed_object::delete_property(analyzer, dep, key)
  }

  fn enumerate_properties(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
  ) -> Vec<(bool, Entity<'a>, Entity<'a>)> {
    self.consume(analyzer);
    consumed_object::enumerate_properties(analyzer, dep)
  }

  fn call(
    &self,
    rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
    this: &Entity<'a>,
    args: &Entity<'a>,
  ) -> Entity<'a> {
    let source = self.dep_node();
    let recursed = analyzer.scope_context.call_scopes.iter().any(|scope| scope.source == source);
    if recursed {
      self.consume(analyzer);
      return consumed_object::call(analyzer, dep, this, args);
    }
    self.call_impl(rc, analyzer, dep, this, args)
  }

  fn r#await(&self, rc: &Entity<'a>, analyzer: &mut Analyzer<'a>) -> (bool, Entity<'a>) {
    if self.consumed.get() {
      return consumed_object::r#await(analyzer);
    }
    (false, rc.clone())
  }

  fn iterate(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
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
    UnknownEntity::new_with_deps(UnknownEntityKind::String, vec![rc.clone()])
  }

  fn get_to_numeric(&self, rc: &Entity<'a>) -> Entity<'a> {
    if self.consumed.get() {
      return consumed_object::get_to_numeric();
    }
    LiteralEntity::new_nan()
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
    variable_scopes: Vec<Rc<RefCell<VariableScope<'a>>>>,
    is_expression: bool,
  ) -> Entity<'a> {
    Entity::new(Self {
      consumed: Rc::new(Cell::new(false)),
      source,
      variable_scopes: Rc::new(variable_scopes),
      is_expression,
    })
  }

  pub fn dep_node(&self) -> EntityDepNode {
    EntityDepNode::from(match self.source {
      FunctionEntitySource::Function(node) => AstKind::Function(node),
      FunctionEntitySource::ArrowFunctionExpression(node) => AstKind::ArrowFunctionExpression(node),
    })
  }

  pub fn dep(&self) -> EntityDep {
    self.dep_node().into()
  }

  pub fn call_impl(
    &self,
    rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
    this: &Entity<'a>,
    args: &Entity<'a>,
  ) -> Entity<'a> {
    let source = self.dep_node();
    let dep = (source, dep).into();
    let variable_scopes = self.variable_scopes.clone();
    let ret_val = match self.source {
      FunctionEntitySource::Function(node) => analyzer.call_function(
        rc.clone(),
        self.dep(),
        source,
        self.is_expression,
        dep,
        node,
        variable_scopes,
        this.clone(),
        args.clone(),
      ),
      FunctionEntitySource::ArrowFunctionExpression(node) => {
        analyzer.call_arrow_function_expression(source, dep, node, variable_scopes, args.clone())
      }
    };
    ForwardedEntity::new(ret_val, self.dep())
  }
}
