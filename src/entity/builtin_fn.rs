use super::{
  consumed_object,
  entity::{EnumeratedProperties, IteratedElements},
  Entity, EntityFactory, EntityTrait, ObjectEntity, TypeofResult,
};
use crate::{
  analyzer::Analyzer,
  consumable::{box_consumable, Consumable},
};
use std::fmt::Debug;

pub trait BuiltinFnEntity<'a>: Debug {
  #[cfg(feature = "flame")]
  fn name(&self) -> &'static str;
  fn object(&self) -> Option<&'a ObjectEntity<'a>> {
    None
  }
  fn call_impl(
    &self,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: Entity<'a>,
    args: Entity<'a>,
  ) -> Entity<'a>;
}

impl<'a, T: BuiltinFnEntity<'a>> EntityTrait<'a> for T {
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    if let Some(object) = self.object() {
      object.consume(analyzer);
    }
  }

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
    if let Some(object) = self.object() {
      object.get_property(rc, analyzer, dep, key)
    } else {
      analyzer.builtins.prototypes.function.get_property(analyzer, rc, key, dep)
    }
  }

  fn set_property(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
    value: Entity<'a>,
  ) {
    if let Some(object) = self.object() {
      object.set_property(rc, analyzer, dep, key, value)
    } else {
      analyzer.add_diagnostic(
      "Should not set property of builtin function, it may cause unexpected tree-shaking behavior",
    );
      consumed_object::set_property(analyzer, dep, key, value)
    }
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, key: Entity<'a>) {
    if let Some(object) = self.object() {
      object.delete_property(analyzer, dep, key)
    } else {
      analyzer.add_diagnostic("Should not delete property of builtin function, it may cause unexpected tree-shaking behavior");
      consumed_object::delete_property(analyzer, dep, key)
    }
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
    #[cfg(feature = "flame")]
    let _scope_guard = flame::start_guard(self.name());
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

  fn get_destructable(
    &self,
    rc: Entity<'a>,
    _analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> Consumable<'a> {
    box_consumable((rc, dep))
  }

  fn get_typeof(&self, _rc: Entity<'a>, analyzer: &mut Analyzer<'a>) -> Entity<'a> {
    analyzer.factory.string("function")
  }

  fn get_to_string(&self, rc: Entity<'a>, analyzer: &mut Analyzer<'a>) -> Entity<'a> {
    analyzer.factory.computed_unknown_string(rc)
  }

  fn get_to_numeric(&self, _rc: Entity<'a>, analyzer: &mut Analyzer<'a>) -> Entity<'a> {
    analyzer.factory.nan
  }

  fn get_to_boolean(&self, _rc: Entity<'a>, analyzer: &mut Analyzer<'a>) -> Entity<'a> {
    analyzer.factory.boolean(true)
  }

  fn get_to_property_key(&self, rc: Entity<'a>, analyzer: &mut Analyzer<'a>) -> Entity<'a> {
    self.get_to_string(rc, analyzer)
  }

  fn get_to_jsx_child(&self, _rc: Entity<'a>, analyzer: &mut Analyzer<'a>) -> Entity<'a> {
    // TODO: analyzer.thrown_builtin_error("Functions are not valid JSX children");
    analyzer.factory.string("")
  }

  fn test_typeof(&self, _analyzer: &mut Analyzer<'a>) -> TypeofResult {
    TypeofResult::Function
  }

  fn test_truthy(&self, _analyzer: &mut Analyzer<'a>) -> Option<bool> {
    Some(true)
  }

  fn test_nullish(&self, _analyzer: &mut Analyzer<'a>) -> Option<bool> {
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
  #[cfg(feature = "flame")]
  name: &'static str,
  implementation: F,
  object: Option<&'a ObjectEntity<'a>>,
}

impl<'a, F: BuiltinFnImplementation<'a> + 'a> Debug for ImplementedBuiltinFnEntity<'a, F> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ImplementedBuiltinFnEntity").finish()
  }
}

impl<'a, F: BuiltinFnImplementation<'a> + 'a> BuiltinFnEntity<'a>
  for ImplementedBuiltinFnEntity<'a, F>
{
  #[cfg(feature = "flame")]
  fn name(&self) -> &'static str {
    self.name
  }
  fn object(&self) -> Option<&'a ObjectEntity<'a>> {
    self.object
  }
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
    name: &'static str,
    implementation: F,
  ) -> Entity<'a> {
    self.entity(ImplementedBuiltinFnEntity {
      #[cfg(feature = "flame")]
      name,
      implementation,
      object: None,
    })
  }
}

impl<'a> Analyzer<'a> {
  pub fn dynamic_implemented_builtin<F: BuiltinFnImplementation<'a> + 'a>(
    &mut self,
    name: &'static str,
    implementation: F,
  ) -> Entity<'a> {
    self.factory.entity(ImplementedBuiltinFnEntity {
      #[cfg(feature = "flame")]
      name,
      implementation,
      object: Some(self.new_function_object()),
    })
  }
}

#[derive(Debug, Clone)]
pub struct PureBuiltinFnEntity<'a> {
  return_value: fn(&EntityFactory<'a>) -> Entity<'a>,
}

impl<'a> BuiltinFnEntity<'a> for PureBuiltinFnEntity<'a> {
  #[cfg(feature = "flame")]
  fn name(&self) -> &'static str {
    "<PureBuiltin>"
  }
  fn call_impl(
    &self,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: Entity<'a>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    let ret_val = (self.return_value)(&analyzer.factory);
    let dep = box_consumable((dep, this, args));
    this.unknown_mutate(analyzer, dep.cloned());
    args.unknown_mutate(analyzer, dep.cloned());
    analyzer.factory.computed(ret_val, dep)
  }
}

impl<'a> PureBuiltinFnEntity<'a> {
  pub fn new(return_value: fn(&EntityFactory<'a>) -> Entity<'a>) -> Self {
    Self { return_value }
  }
}
