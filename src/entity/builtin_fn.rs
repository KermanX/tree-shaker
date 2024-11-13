use super::{
  consumed_object,
  entity::{EnumeratedProperties, IteratedElements},
  Entity, EntityFactory, EntityTrait, TypeofResult,
};
use crate::{
  analyzer::Analyzer,
  consumable::{box_consumable, Consumable},
};
use std::{fmt::Debug, marker::PhantomData};

pub trait BuiltinFnEntity<'a>: Debug {
  fn call_impl(
    &self,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: Entity<'a>,
    args: Entity<'a>,
  ) -> Entity<'a>;
}

impl<'a, T: BuiltinFnEntity<'a>> EntityTrait<'a> for T {
  fn consume(&self, _analyzer: &mut Analyzer<'a>) {}

  fn unknown_mutate(&self, _analyzer: &mut Analyzer<'a>, _dep: Consumable<'a>) {
    // No effect
  }

  fn get_property(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
  ) -> Entity<'a> {
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
    analyzer.add_diagnostic(
      "Should not set property of builtin function, it may cause unexpected tree-shaking behavior",
    );
    consumed_object::set_property(analyzer, dep, key, value)
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, key: Entity<'a>) {
    analyzer.add_diagnostic("Should not delete property of builtin function, it may cause unexpected tree-shaking behavior");
    consumed_object::delete_property(analyzer, dep, key)
  }

  fn enumerate_properties(
    &self,
    _rc: Entity<'a>,
    _analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> EnumeratedProperties<'a> {
    (vec![], dep)
  }

  fn call(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: Entity<'a>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    self.call_impl(analyzer, dep, this, args)
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

  fn jsx(&self, _rc: Entity<'a>, analyzer: &mut Analyzer<'a>, props: Entity<'a>) -> Entity<'a> {
    self.call_impl(
      analyzer,
      box_consumable(()),
      analyzer.factory.immutable_unknown,
      analyzer.factory.arguments(vec![(false, props)]),
    )
  }

  fn r#await(
    &self,
    rc: Entity<'a>,
    _analyzer: &mut Analyzer<'a>,
    _dep: Consumable<'a>,
  ) -> Entity<'a> {
    rc
  }

  fn iterate(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> IteratedElements<'a> {
    analyzer.thrown_builtin_error("Cannot iterate over function");
    consumed_object::iterate(analyzer, dep)
  }

  fn get_destructable(&self, rc: Entity<'a>, dep: Consumable<'a>) -> Consumable<'a> {
    box_consumable((rc, dep))
  }

  fn get_typeof(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    analyzer.factory.string("function")
  }

  fn get_to_string(&self, rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    analyzer.factory.computed_unknown_string(rc)
  }

  fn get_to_numeric(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    analyzer.factory.nan
  }

  fn get_to_boolean(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    analyzer.factory.boolean(true)
  }

  fn get_to_property_key(&self, rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    self.get_to_string(rc, analyzer)
  }

  fn get_to_jsx_child(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    // TODO: analyzer.thrown_builtin_error("Functions are not valid JSX children");
    analyzer.factory.string("")
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

pub trait BuiltinFnImplementation<'a>:
  Fn(&mut Analyzer<'a>, Consumable<'a>, Entity<'a>, Entity<'a>) -> Entity<'a>
{
}
impl<'a, T: Fn(&mut Analyzer<'a>, Consumable<'a>, Entity<'a>, Entity<'a>) -> Entity<'a>>
  BuiltinFnImplementation<'a> for T
{
}

#[derive(Clone, Copy)]
pub struct ImplementedBuiltinFnEntity<'a, F: BuiltinFnImplementation<'a> + 'a> {
  implementation: F,
  phantom_data: PhantomData<&'a ()>,
}

impl<'a, F: BuiltinFnImplementation<'a> + 'a> Debug for ImplementedBuiltinFnEntity<'a, F> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ImplementedBuiltinFnEntity").finish()
  }
}

impl<'a, F: BuiltinFnImplementation<'a> + 'a> BuiltinFnEntity<'a>
  for ImplementedBuiltinFnEntity<'a, F>
{
  fn call_impl(
    &self,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: Entity<'a>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    (self.implementation)(analyzer, dep, this, args)
  }
}

impl<'a> EntityFactory<'a> {
  pub fn implemented_builtin_fn<F: BuiltinFnImplementation<'a> + 'a>(
    &self,
    implementation: F,
  ) -> Entity<'a> {
    self.entity(ImplementedBuiltinFnEntity { implementation, phantom_data: PhantomData })
  }
}

#[derive(Debug, Clone)]
pub struct PureBuiltinFnEntity<'a> {
  return_value: fn(&EntityFactory<'a>) -> Entity<'a>,
}

impl<'a> BuiltinFnEntity<'a> for PureBuiltinFnEntity<'a> {
  fn call_impl(
    &self,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: Entity<'a>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    let ret_val = (self.return_value)(&analyzer.factory);
    let dep = box_consumable((dep, this, args));
    if analyzer.is_inside_pure() {
      this.unknown_mutate(analyzer, dep.cloned());
      args.unknown_mutate(analyzer, dep.cloned());
      analyzer.factory.computed(ret_val, dep)
    } else {
      analyzer.consume(dep);
      ret_val
    }
  }
}

impl<'a> PureBuiltinFnEntity<'a> {
  pub fn new(return_value: fn(&EntityFactory<'a>) -> Entity<'a>) -> Self {
    Self { return_value }
  }
}
