use super::{
  consumed_object,
  entity::{EnumeratedProperties, IteratedElements},
  Entity, EntityTrait, ObjectEntity, TypeofResult,
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
  pub finite_recursion: bool,
  pub object: &'a ObjectEntity<'a>,
}

impl<'a> EntityTrait<'a> for FunctionEntity<'a> {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    use_consumed_flag!(self);

    self.consume_body(analyzer);

    self.object.consume(analyzer);
  }

  fn unknown_mutate(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>) {
    self.consume(analyzer);
    consumed_object::unknown_mutate(analyzer, dep);
  }

  fn get_property(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
  ) -> Entity<'a> {
    self.object.get_property(rc, analyzer, self.forward_dep(dep), key)
  }

  fn set_property(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
    value: Entity<'a>,
  ) {
    self.object.set_property(rc, analyzer, self.forward_dep(dep), key, value);
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, key: Entity<'a>) {
    self.object.delete_property(analyzer, self.forward_dep(dep), key);
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

    let maybe_infinite_recursion = !self.finite_recursion
      && analyzer.scope_context.call.iter().any(|scope| scope.callee.1 == self.callee.1);
    if maybe_infinite_recursion {
      self.consume_body(analyzer);
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

  fn jsx(&self, rc: Entity<'a>, analyzer: &mut Analyzer<'a>, props: Entity<'a>) -> Entity<'a> {
    self.call(
      rc,
      analyzer,
      box_consumable(()),
      analyzer.factory.immutable_unknown,
      analyzer.factory.arguments(vec![(false, props)]),
    )
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

  fn get_to_jsx_child(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    if self.consumed.get() {
      analyzer.factory.immutable_unknown
    } else {
      // TODO: analyzer.thrown_builtin_error("Functions are not valid JSX children");
      analyzer.factory.string("")
    }
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

  fn forward_dep(&self, dep: Consumable<'a>) -> Consumable<'a> {
    box_consumable((dep, self.callee.0.into_dep_id()))
  }
}

impl<'a> Analyzer<'a> {
  pub fn new_function(&mut self, node: CalleeNode<'a>) -> Entity<'a> {
    let function = FunctionEntity {
      consumed: Rc::new(Cell::new(false)),
      body_consumed: Rc::new(Cell::new(false)),
      callee: (node, self.factory.alloc_instance_id()),
      variable_scope_stack: Rc::new(self.scope_context.variable.stack.clone()),
      finite_recursion: self.has_finite_recursion_notation(node.span()),
      object: self.allocator.alloc(self.new_empty_object(&self.builtins.prototypes.function)),
    };

    let mut created_in_self = false;
    for scope in self.scope_context.call.iter().rev() {
      if scope.callee.0 == node {
        created_in_self = true;
        break;
      }
    }

    if created_in_self {
      function.consume_body(self);
      self.factory.unknown()
    } else {
      self.factory.entity(function)
    }
  }
}
