use super::{
  consumed_object,
  entity::{EnumeratedProperties, IteratedElements},
  Entity, EntityTrait, TypeofResult,
};
use crate::{
  analyzer::Analyzer,
  consumable::{box_consumable, Consumable},
  scope::call_scope::CalleeNode,
  use_consumed_flag,
};
use oxc::{semantic::ScopeId, span::GetSpan};
use std::{cell::Cell, rc::Rc};

#[derive(Debug, Clone)]
pub struct FunctionEntity<'a> {
  consumed: Rc<Cell<bool>>,
  body_consumed: Rc<Cell<bool>>,
  pub callee: (CalleeNode<'a>, usize),
  pub variable_scope_stack: Rc<Vec<ScopeId>>,
}

impl<'a> EntityTrait<'a> for FunctionEntity<'a> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    use_consumed_flag!(self);

    self.consume_body(analyzer);
  }

  fn unknown_mutate(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>) {
    if self.consumed.get() {
      return;
    }

    analyzer.push_dependent_cf_scope(dep);
    self.consume_body(analyzer);
    analyzer.pop_cf_scope();
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

    let recursed = analyzer.scope_context.call.iter().any(|scope| scope.callee.1 == self.callee.1);
    if recursed {
      self.consume_body(analyzer);
      return analyzer.factory.unknown();
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
      logger.push_fn_call(self.callee.0.span(), self.callee.0.name());
    }

    let call_dep = box_consumable((self.callee.0.into_dep_id(), dep));
    let variable_scopes = self.variable_scope_stack.clone();
    let ret_val = match self.callee.0 {
      CalleeNode::Function(node) => analyzer.call_function(
        rc,
        self.callee,
        call_dep.cloned(),
        node,
        variable_scopes,
        this.clone(),
        args.clone(),
        consume,
      ),
      CalleeNode::ArrowFunctionExpression(node) => analyzer.call_arrow_function_expression(
        self.callee,
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

  pub fn consume_body(&self, analyzer: &mut Analyzer<'a>) {
    if self.body_consumed.get() {
      return;
    }
    self.body_consumed.set(true);

    analyzer.consume(self.callee.0.into_dep_id());
    analyzer.consume_arguments(Some(self.callee));

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

impl<'a> Analyzer<'a> {
  pub fn new_function(&mut self, node: CalleeNode<'a>) -> Entity<'a> {
    let function = FunctionEntity {
      consumed: Rc::new(Cell::new(false)),
      body_consumed: Rc::new(Cell::new(false)),
      callee: (node, self.factory.alloc_instance_id()),
      variable_scope_stack: Rc::new(self.scope_context.variable.stack.clone()),
    };

    let mut recursed = false;
    for scope in self.scope_context.call.iter().rev() {
      if scope.callee.0 == node {
        recursed = true;
        break;
      }
    }

    if recursed {
      function.consume_body(self);
      self.factory.unknown()
    } else {
      self.factory.entity(function)
    }
  }
}
