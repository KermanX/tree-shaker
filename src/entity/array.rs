use super::{
  consumed_object,
  entity::{EnumeratedProperties, IteratedElements},
  Entity, EntityTrait, LiteralEntity, TypeofResult,
};
use crate::{
  analyzer::Analyzer,
  consumable::{box_consumable, Consumable, ConsumableCollector, ConsumableNode, ConsumableTrait},
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
  fn consume(&self, analyzer: &mut Analyzer<'a>) {
    use_consumed_flag!(self);

    analyzer.mark_object_consumed(self.cf_scope, self.object_id);

    self.deps.take().consume_all(analyzer);

    for element in self.elements.borrow().iter() {
      element.consume(analyzer);
    }

    for rest in self.rest.borrow().iter() {
      rest.consume(analyzer);
    }
  }

  fn unknown_mutate(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>) {
    if self.consumed.get() {
      return consumed_object::unknown_mutate(analyzer, dep);
    }

    let (has_exhaustive, _, exec_deps) = analyzer.pre_mutate_object(self.cf_scope, self.object_id);

    if has_exhaustive {
      self.consume(analyzer);
      return consumed_object::unknown_mutate(analyzer, dep);
    }

    self.deps.borrow_mut().push(box_consumable((exec_deps, dep)));
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

    analyzer.mark_object_property_exhaustive_read(self.cf_scope, self.object_id);

    if !self.deps.borrow().is_empty() {
      return analyzer.factory.computed_unknown((rc, dep, key));
    }

    let dep = ConsumableNode::new((self.deps.borrow_mut().collect(), dep, key.clone()));
    let key = key.get_to_property_key(analyzer);
    if let Some(key_literals) = key.get_to_literals(analyzer) {
      let mut result = vec![];
      let mut rest_added = false;
      let mut undefined_added = false;
      for key_literal in key_literals {
        match key_literal {
          LiteralEntity::String(key, _) => {
            if let Some(index) = key.parse::<usize>().ok() {
              if let Some(element) = self.elements.borrow().get(index) {
                result.push(*element);
              } else if !rest_added {
                rest_added = true;
                result.extend(self.rest.borrow().iter().copied());
                if !undefined_added {
                  undefined_added = true;
                  result.push(analyzer.factory.undefined);
                }
              }
            } else if key == "length" {
              result.push(self.get_length().map_or_else(
                || {
                  let dep = ConsumableNode::new(self.rest.borrow().clone());
                  analyzer.factory.computed_unknown_number(dep)
                },
                |length| analyzer.factory.number(length as f64, None),
              ));
            } else if let Some(property) = analyzer.builtins.prototypes.array.get_string_keyed(key)
            {
              result.push(property.clone());
            } else if !undefined_added {
              undefined_added = true;
              result.push(analyzer.factory.undefined);
            }
          }
          LiteralEntity::Symbol(key, _) => todo!(),
          _ => unreachable!("Invalid property key"),
        }
      }
      analyzer.factory.computed_union(result, dep)
    } else {
      analyzer.factory.computed_unknown((
        ConsumableNode::new(
          self
            .elements
            .borrow()
            .iter()
            .chain(self.rest.borrow().iter())
            .cloned()
            .collect::<Vec<_>>(),
        ),
        dep,
      ))
    }
  }

  fn set_property(
    &self,
    _rc: Entity<'a>,
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
            if let Some(index) = key_str.parse::<usize>().ok() {
              has_effect = true;
              if let Some(element) = self.elements.borrow_mut().get_mut(index) {
                *element = if definite {
                  value.clone()
                } else {
                  analyzer.factory.union((element.clone(), value.clone()))
                };
              } else if !rest_added {
                rest_added = true;
                self.rest.borrow_mut().push(value.clone());
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
        self.add_assignment_dep(exec_deps, dep);
      }
      return;
    }

    // Unknown
    let mut deps = self.deps.borrow_mut();
    deps.push(dep);
    deps.push(box_consumable((exec_deps, key.get_to_property_key(analyzer), value)));
  }

  fn enumerate_properties(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> EnumeratedProperties<'a> {
    if self.consumed.get() {
      return consumed_object::enumerate_properties(rc, analyzer, dep);
    }

    analyzer.mark_object_property_exhaustive_read(self.cf_scope, self.object_id);

    if !self.deps.borrow().is_empty() {
      return (
        vec![(false, analyzer.factory.unknown_primitive, analyzer.factory.unknown())],
        box_consumable((rc.clone(), dep)),
      );
    }

    let mut entries = Vec::new();
    for (i, element) in self.elements.borrow().iter().enumerate() {
      entries.push((
        true,
        analyzer.factory.string(analyzer.allocator.alloc(i.to_string())),
        element.clone(),
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

    (entries, box_consumable((self.deps.borrow_mut().collect(), dep.cloned())))
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, key: Entity<'a>) {
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
    deps.push(box_consumable((exec_deps, key.get_to_property_key(analyzer))));
  }

  fn call(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: Entity<'a>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    consumed_object::call(rc, analyzer, dep, this, args)
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
    consumed_object::jsx(rc, analyzer, props)
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
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> IteratedElements<'a> {
    if self.consumed.get() {
      return consumed_object::iterate(analyzer, dep);
    }

    if !self.deps.borrow().is_empty() {
      return (vec![], Some(analyzer.factory.unknown()), box_consumable((rc, dep)));
    }

    (
      self.elements.borrow().clone(),
      analyzer.factory.try_union(self.rest.borrow().clone()),
      box_consumable((dep, self.deps.borrow_mut().collect())),
    )
  }

  fn get_destructable(&self, _rc: Entity<'a>, dep: Consumable<'a>) -> Consumable<'a> {
    dep
  }

  fn get_typeof(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    analyzer.factory.string("object")
  }

  fn get_to_string(&self, rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    if self.consumed.get() {
      return consumed_object::get_to_string(analyzer);
    }
    analyzer.factory.computed_unknown_string(rc)
  }

  fn get_to_numeric(&self, rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    if self.consumed.get() {
      return consumed_object::get_to_numeric(analyzer);
    }
    analyzer.factory.computed_unknown(rc)
  }

  fn get_to_boolean(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    analyzer.factory.boolean(true)
  }

  fn get_to_property_key(&self, rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    self.get_to_string(rc, analyzer)
  }

  fn get_to_jsx_child(&self, rc: Entity<'a>, _analyzer: &Analyzer<'a>) -> Entity<'a> {
    rc
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
  pub fn new(cf_scope: ScopeId, object_id: SymbolId) -> Self {
    ArrayEntity {
      consumed: Cell::new(false),
      deps: Default::default(),
      cf_scope,
      object_id,
      elements: RefCell::new(Vec::new()),
      rest: RefCell::new(Vec::new()),
    }
  }

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

  fn add_assignment_dep<T: ConsumableTrait<'a> + 'a>(
    &self,
    exec_deps: ConsumableNode<'a, T>,
    dep: impl ConsumableTrait<'a> + 'a,
  ) {
    let mut deps = self.deps.borrow_mut();
    deps.push(box_consumable((exec_deps, dep)));
  }
}

impl<'a> Analyzer<'a> {
  pub fn new_empty_array(&mut self) -> ArrayEntity<'a> {
    ArrayEntity::new(self.scope_context.cf.current_id(), self.scope_context.alloc_object_id())
  }
}
