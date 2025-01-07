use super::{
  consumed_object,
  entity::{EnumeratedProperties, IteratedElements},
  Entity, EntityFactory, EntityTrait, TypeofResult,
};
use crate::{analyzer::Analyzer, consumable::Consumable, use_consumed_flag};
use std::cell::{Cell, RefCell};

#[derive(Debug)]
pub struct ReactElementEntity<'a> {
  consumed: Cell<bool>,
  tag: Entity<'a>,
  props: Entity<'a>,
  deps: RefCell<Vec<Consumable<'a>>>,
}

impl<'a> EntityTrait<'a> for ReactElementEntity<'a> {
  fn consume(&'a self, analyzer: &mut Analyzer<'a>) {
    use_consumed_flag!(self);

    analyzer.consume(self.deps.take());

    let tag = self.tag;
    let props = self.props;
    // Is this the best way to handle this?
    let group_id = analyzer.new_object_mangling_group();
    analyzer.exec_consumed_fn("React_blackbox", move |analyzer| {
      let copied_props =
        analyzer.new_empty_object(&analyzer.builtins.prototypes.object, Some(group_id));
      copied_props.init_spread(analyzer, analyzer.factory.empty_consumable, props);
      tag.jsx(analyzer, copied_props)
    });
  }

  fn unknown_mutate(&'a self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>) {
    if self.consumed.get() {
      return consumed_object::unknown_mutate(analyzer, dep);
    }

    self.deps.borrow_mut().push(dep);
  }

  fn get_property(
    &'a self,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
  ) -> Entity<'a> {
    consumed_object::get_property(self, analyzer, dep, key)
  }

  fn set_property(
    &'a self,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
    value: Entity<'a>,
  ) {
    self.consume(analyzer);
    consumed_object::set_property(analyzer, dep, key, value)
  }

  fn enumerate_properties(
    &'a self,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> EnumeratedProperties<'a> {
    if analyzer.config.unknown_property_read_side_effects {
      self.consume(analyzer);
    }
    consumed_object::enumerate_properties(self, analyzer, dep)
  }

  fn delete_property(&'a self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, key: Entity<'a>) {
    self.consume(analyzer);
    consumed_object::delete_property(analyzer, dep, key)
  }

  fn call(
    &'a self,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: Entity<'a>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    analyzer.thrown_builtin_error("Cannot call a React element");
    consumed_object::call(self, analyzer, dep, this, args)
  }

  fn construct(
    &'a self,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    analyzer.thrown_builtin_error("Cannot call a React element");
    consumed_object::construct(self, analyzer, dep, args)
  }

  fn jsx(&'a self, analyzer: &mut Analyzer<'a>, props: Entity<'a>) -> Entity<'a> {
    analyzer.thrown_builtin_error("Cannot call a React element");
    consumed_object::jsx(self, analyzer, props)
  }

  fn r#await(&'a self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>) -> Entity<'a> {
    self.consume(analyzer);
    consumed_object::r#await(analyzer, dep)
  }

  fn iterate(&'a self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>) -> IteratedElements<'a> {
    self.consume(analyzer);
    consumed_object::iterate(analyzer, dep)
  }

  fn get_destructable(&'a self, analyzer: &Analyzer<'a>, dep: Consumable<'a>) -> Consumable<'a> {
    analyzer.consumable((self, dep))
  }

  fn get_typeof(&'a self, analyzer: &Analyzer<'a>) -> Entity<'a> {
    analyzer.factory.computed_unknown_string(self)
  }

  fn get_to_string(&'a self, analyzer: &Analyzer<'a>) -> Entity<'a> {
    analyzer.factory.computed_unknown_string(self)
  }

  fn get_to_numeric(&'a self, _analyzer: &Analyzer<'a>) -> Entity<'a> {
    self
  }

  fn get_to_boolean(&'a self, analyzer: &Analyzer<'a>) -> Entity<'a> {
    match self.test_truthy() {
      Some(val) => analyzer.factory.boolean(val),
      None => analyzer.factory.unknown_boolean,
    }
  }

  fn get_to_property_key(&'a self, _analyzer: &Analyzer<'a>) -> Entity<'a> {
    self
  }

  fn get_to_jsx_child(&'a self, _analyzer: &Analyzer<'a>) -> Entity<'a> {
    self
  }

  fn test_typeof(&self) -> TypeofResult {
    TypeofResult::_Unknown
  }

  fn test_truthy(&self) -> Option<bool> {
    None
  }

  fn test_nullish(&self) -> Option<bool> {
    None
  }
}

impl<'a> EntityFactory<'a> {
  pub fn react_element(
    &self,
    tag: Entity<'a>,
    props: Entity<'a>,
  ) -> &'a mut ReactElementEntity<'a> {
    self.alloc(ReactElementEntity {
      consumed: Cell::new(false),
      tag,
      props,
      deps: RefCell::new(vec![]),
    })
  }
}
