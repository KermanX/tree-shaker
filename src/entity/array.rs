use super::{
  consumed_object,
  entity::{EnumeratedProperties, IteratedElements},
  Entity, EntityFactory, EntityTrait, LiteralEntity, TypeofResult,
};
use crate::{
  analyzer::Analyzer,
  consumable::{Consumable, ConsumableCollector},
  use_consumed_flag,
};
use oxc::semantic::{ScopeId, SymbolId};
use std::{
  cell::{Cell, RefCell},
  fmt,
};

pub struct ArrayEntity<'a> {
  consumed: Cell<bool>,
  pub deps: RefCell<ConsumableCollector<'a>>,
  cf_scope: ScopeId,
  object_id: SymbolId,
  pub elements: RefCell<Vec<Entity<'a>>>,
  pub rest: RefCell<Vec<Entity<'a>>>,
}

impl<'a> fmt::Debug for ArrayEntity<'a> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("ArrayEntity")
      .field("consumed", &self.consumed.get())
      .field("deps", &self.deps.borrow())
      .field("elements", &self.elements.borrow())
      .field("rest", &self.rest.borrow())
      .finish()
  }
}

impl<'a> EntityTrait<'a> for ArrayEntity<'a> {
  fn consume(&'a self, analyzer: &mut Analyzer<'a>) {
    use_consumed_flag!(self);

    analyzer.mark_object_consumed(self.cf_scope, self.object_id);

    self.deps.take().consume_all(analyzer);

    analyzer.consume(self.elements.take());
    analyzer.consume(self.rest.take());
  }

  fn unknown_mutate(&'a self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>) {
    if self.consumed.get() {
      return consumed_object::unknown_mutate(analyzer, dep);
    }

    let (has_exhaustive, _, exec_deps) = analyzer.pre_mutate_object(self.cf_scope, self.object_id);

    if has_exhaustive {
      self.consume(analyzer);
      return consumed_object::unknown_mutate(analyzer, dep);
    }

    self.deps.borrow_mut().push(analyzer.consumable((exec_deps, dep)));
  }

  fn get_property(
    &'a self,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
  ) -> Entity<'a> {
    if self.consumed.get() {
      return consumed_object::get_property(self, analyzer, dep, key);
    }

    analyzer.mark_object_property_exhaustive_read(self.cf_scope, self.object_id);

    if !self.deps.borrow().is_empty() {
      return analyzer.factory.computed_unknown((self, dep, key));
    }

    let dep = analyzer.consumable((self.deps.borrow_mut().collect(analyzer.factory), dep, key));
    let key = key.get_to_property_key(analyzer);
    if let Some(key_literals) = key.get_to_literals(analyzer) {
      let mut result = vec![];
      let mut rest_added = false;
      for key_literal in key_literals {
        match key_literal {
          LiteralEntity::String(key, _) => {
            if let Ok(index) = key.parse::<usize>() {
              if let Some(element) = self.elements.borrow().get(index) {
                result.push(*element);
              } else if !rest_added {
                rest_added = true;
                result.extend(self.rest.borrow().iter().copied());
                result.push(analyzer.factory.undefined);
              }
            } else if key == "length" {
              result.push(self.get_length().map_or_else(
                || {
                  let dep = self.rest.borrow().clone();
                  analyzer.factory.computed_unknown_number(analyzer.consumable(dep))
                },
                |length| analyzer.factory.number(length as f64, None),
              ));
            } else if let Some(property) = analyzer.builtins.prototypes.array.get_string_keyed(key)
            {
              result.push(property);
            } else {
              result.push(analyzer.factory.unmatched_prototype_property);
            }
          }
          LiteralEntity::Symbol(key, _) => todo!(),
          _ => unreachable!("Invalid property key"),
        }
      }
      analyzer.factory.computed_union(result, dep)
    } else {
      analyzer.factory.computed_unknown(analyzer.consumable((
        self.elements.borrow().iter().chain(self.rest.borrow().iter()).cloned().collect::<Vec<_>>(),
        dep,
      )))
    }
  }

  fn set_property(
    &'a self,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
    value: Entity<'a>,
  ) {
    if self.consumed.get() {
      return consumed_object::set_property(analyzer, dep, key, value);
    }

    let (has_exhaustive, indeterminate, exec_deps) =
      analyzer.pre_mutate_object(self.cf_scope, self.object_id);

    if has_exhaustive {
      self.consume(analyzer);
      return consumed_object::set_property(analyzer, dep, key, value);
    }

    let mut has_effect = false;
    'known: {
      if !self.deps.borrow().is_empty() {
        break 'known;
      }

      let Some(key_literals) = key.get_to_property_key(analyzer).get_to_literals(analyzer) else {
        break 'known;
      };

      let definite = !indeterminate && key_literals.len() == 1;
      let mut rest_added = false;
      for key_literal in key_literals {
        match key_literal {
          LiteralEntity::String(key_str, _) => {
            if let Ok(index) = key_str.parse::<usize>() {
              has_effect = true;
              if let Some(element) = self.elements.borrow_mut().get_mut(index) {
                *element = if definite { value } else { analyzer.factory.union((*element, value)) };
              } else if !rest_added {
                rest_added = true;
                self.rest.borrow_mut().push(value);
              }
            } else if key_str == "length" {
              if let Some(length) = value.get_literal(analyzer).and_then(|lit| lit.to_number()) {
                if let Some(length) = length.map(|l| l.0.trunc()) {
                  let length = length as usize;
                  let mut elements = self.elements.borrow_mut();
                  let mut rest = self.rest.borrow_mut();
                  if elements.len() > length {
                    has_effect = true;
                    elements.truncate(length);
                    rest.clear();
                  } else if !rest.is_empty() {
                    has_effect = true;
                    rest.push(analyzer.factory.undefined);
                  } else if elements.len() < length {
                    has_effect = true;
                    for _ in elements.len()..length {
                      elements.push(analyzer.factory.undefined);
                    }
                  }
                } else {
                  analyzer.thrown_builtin_error("Invalid array length");
                  has_effect = true;
                }
              } else {
                has_effect = true;
              }
            } else {
              break 'known;
            }
          }
          LiteralEntity::Symbol(key, _) => todo!(),
          _ => unreachable!("Invalid property key"),
        }
      }
      if has_effect {
        let mut deps = self.deps.borrow_mut();
        deps.push(analyzer.consumable((exec_deps, dep)));
      }
      return;
    }

    // Unknown
    let mut deps = self.deps.borrow_mut();
    deps.push(dep);
    deps.push(analyzer.consumable((exec_deps, key.get_to_property_key(analyzer), value)));
  }

  fn enumerate_properties(
    &'a self,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> EnumeratedProperties<'a> {
    if self.consumed.get() {
      return consumed_object::enumerate_properties(self, analyzer, dep);
    }

    analyzer.mark_object_property_exhaustive_read(self.cf_scope, self.object_id);

    if !self.deps.borrow().is_empty() {
      return (
        vec![(false, analyzer.factory.unknown_primitive, analyzer.factory.unknown())],
        analyzer.consumable((self, dep)),
      );
    }

    let mut entries = Vec::new();
    for (i, element) in self.elements.borrow().iter().enumerate() {
      entries.push((
        true,
        analyzer.factory.string(analyzer.allocator.alloc(i.to_string())),
        *element,
      ));
    }
    let rest = self.rest.borrow();
    if !rest.is_empty() {
      entries.push((
        true,
        analyzer.factory.unknown_string,
        analyzer.factory.union(rest.iter().cloned().collect::<Vec<_>>()),
      ));
    }

    (entries, analyzer.consumable((self.deps.borrow_mut().collect(analyzer.factory), dep)))
  }

  fn delete_property(&'a self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, key: Entity<'a>) {
    if self.consumed.get() {
      return consumed_object::delete_property(analyzer, dep, key);
    }

    let (has_exhaustive, _, exec_deps) = analyzer.pre_mutate_object(self.cf_scope, self.object_id);

    if has_exhaustive {
      self.consume(analyzer);
      return consumed_object::delete_property(analyzer, dep, key);
    }

    let mut deps = self.deps.borrow_mut();
    deps.push(dep);
    deps.push(analyzer.consumable((exec_deps, key.get_to_property_key(analyzer))));
  }

  fn call(
    &'a self,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: Entity<'a>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    consumed_object::call(self, analyzer, dep, this, args)
  }

  fn construct(
    &'a self,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    consumed_object::construct(self, analyzer, dep, args)
  }

  fn jsx(&'a self, analyzer: &mut Analyzer<'a>, props: Entity<'a>) -> Entity<'a> {
    consumed_object::jsx(self, analyzer, props)
  }

  fn r#await(&'a self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>) -> Entity<'a> {
    if self.consumed.get() {
      return consumed_object::r#await(analyzer, dep);
    }
    analyzer.factory.computed(self, dep)
  }

  fn iterate(&'a self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>) -> IteratedElements<'a> {
    if self.consumed.get() {
      return consumed_object::iterate(analyzer, dep);
    }

    analyzer.mark_object_property_exhaustive_read(self.cf_scope, self.object_id);

    if !self.deps.borrow().is_empty() {
      return (vec![], Some(analyzer.factory.unknown()), analyzer.consumable((self, dep)));
    }

    (
      self.elements.borrow().clone(),
      analyzer.factory.try_union(self.rest.borrow().clone()),
      analyzer.consumable((dep, self.deps.borrow_mut().collect(analyzer.factory))),
    )
  }

  fn get_destructable(&'a self, _analyzer: &Analyzer<'a>, dep: Consumable<'a>) -> Consumable<'a> {
    dep
  }

  fn get_typeof(&'a self, analyzer: &Analyzer<'a>) -> Entity<'a> {
    analyzer.factory.string("object")
  }

  fn get_to_string(&'a self, analyzer: &Analyzer<'a>) -> Entity<'a> {
    if self.consumed.get() {
      return consumed_object::get_to_string(analyzer);
    }
    analyzer.factory.computed_unknown_string(self)
  }

  fn get_to_numeric(&'a self, analyzer: &Analyzer<'a>) -> Entity<'a> {
    if self.consumed.get() {
      return consumed_object::get_to_numeric(analyzer);
    }
    analyzer.factory.computed_unknown(self)
  }

  fn get_to_boolean(&'a self, analyzer: &Analyzer<'a>) -> Entity<'a> {
    analyzer.factory.boolean(true)
  }

  fn get_to_property_key(&'a self, analyzer: &Analyzer<'a>) -> Entity<'a> {
    self.get_to_string(analyzer)
  }

  fn get_to_jsx_child(&'a self, _analyzer: &Analyzer<'a>) -> Entity<'a> {
    self
  }

  fn test_typeof(&self) -> TypeofResult {
    TypeofResult::Object
  }

  fn test_truthy(&self) -> Option<bool> {
    Some(true)
  }

  fn test_nullish(&self) -> Option<bool> {
    Some(false)
  }
}

impl<'a> ArrayEntity<'a> {
  pub fn push_element(&self, element: Entity<'a>) {
    if self.rest.borrow().is_empty() {
      self.elements.borrow_mut().push(element);
    } else {
      self.init_rest(element);
    }
  }

  pub fn init_rest(&self, rest: Entity<'a>) {
    self.rest.borrow_mut().push(rest);
  }

  pub fn get_length(&self) -> Option<usize> {
    if self.rest.borrow().is_empty() {
      Some(self.elements.borrow().len())
    } else {
      None
    }
  }
}

impl<'a> EntityFactory<'a> {
  pub fn array(
    &self,
    cf_scope: ScopeId,
    object_id: SymbolId,
    elements: Vec<Entity<'a>>,
    rest: Vec<Entity<'a>>,
  ) -> &'a mut ArrayEntity<'a> {
    self.alloc(ArrayEntity {
      consumed: Cell::new(false),
      deps: Default::default(),
      cf_scope,
      object_id,
      elements: RefCell::new(elements),
      rest: RefCell::new(rest),
    })
  }

  pub fn empty_array(&self, cf_scope: ScopeId, object_id: SymbolId) -> &'a mut ArrayEntity<'a> {
    self.array(cf_scope, object_id, Vec::new(), Vec::new())
  }
}

impl<'a> Analyzer<'a> {
  pub fn new_array(
    &mut self,
    elements: Vec<Entity<'a>>,
    rest: Vec<Entity<'a>>,
  ) -> &'a mut ArrayEntity<'a> {
    self.factory.array(
      self.scope_context.cf.current_id(),
      self.scope_context.alloc_object_id(),
      elements,
      rest,
    )
  }

  pub fn new_empty_array(&mut self) -> &'a mut ArrayEntity<'a> {
    self.new_array(Vec::new(), Vec::new())
  }
}
