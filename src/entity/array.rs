use super::{
  consumed_object, ComputedEntity, Entity, EntityTrait, ForwardedEntity, LiteralEntity,
  TypeofResult, UnionEntity, UnknownEntity,
};
use crate::{
  analyzer::Analyzer,
  consumable::{box_consumable, Consumable, ConsumableCollector, ConsumableNode},
  use_consumed_flag,
};
use oxc::{semantic::ScopeId, syntax::number::ToJsInt32};
use std::{
  cell::{Cell, RefCell},
  fmt,
};

pub struct ArrayEntity<'a> {
  consumed: Cell<bool>,
  pub deps: RefCell<ConsumableCollector<'a>>,
  cf_scope: ScopeId,
  variable_scope: ScopeId,
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

    analyzer.refer_to_diff_variable_scope(self.variable_scope);

    self.deps.borrow_mut().consume_all(analyzer);

    for element in self.elements.borrow().iter() {
      element.consume(analyzer);
    }

    for rest in self.rest.borrow().iter() {
      rest.consume(analyzer);
    }
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
    let dep = ConsumableNode::new_box((self.deps.borrow_mut().collect(), dep, key.clone()));
    let key = key.get_to_property_key();
    if let Some(key_literals) = key.get_to_literals() {
      let mut result = vec![];
      let mut rest_added = false;
      let mut undefined_added = false;
      for key_literal in key_literals {
        match key_literal {
          LiteralEntity::String(key) => {
            if let Some(index) = key.parse::<usize>().ok() {
              if let Some(element) = self.elements.borrow().get(index) {
                result.push(element.clone());
              } else if !rest_added {
                rest_added = true;
                result.extend(self.rest.borrow().iter().cloned());
                if !undefined_added {
                  undefined_added = true;
                  result.push(LiteralEntity::new_undefined());
                }
              }
            } else if key == "length" {
              result.push(self.get_length().map_or_else(
                || {
                  let dep: Vec<_> = self.rest.borrow().iter().cloned().collect();
                  UnknownEntity::new_computed_number(box_consumable(dep))
                },
                |length| {
                  LiteralEntity::new_number(
                    length as f64,
                    analyzer.allocator.alloc(length.to_string()),
                  )
                },
              ));
            } else if let Some(property) = analyzer.builtins.prototypes.array.get(key) {
              result.push(property.clone());
            } else if !undefined_added {
              undefined_added = true;
              result.push(LiteralEntity::new_undefined());
            }
          }
          LiteralEntity::Symbol(key, _) => todo!(),
          _ => unreachable!(),
        }
      }
      UnionEntity::new_computed(result, box_consumable(dep))
    } else {
      UnknownEntity::new_computed_unknown(box_consumable((
        ConsumableNode::new_box(
          self
            .elements
            .borrow()
            .iter()
            .chain(self.rest.borrow().iter())
            .cloned()
            .collect::<Vec<_>>(),
        ),
        dep,
      )))
    }
  }

  fn set_property(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: &Entity<'a>,
    value: Entity<'a>,
  ) {
    if self.consumed.get() {
      return consumed_object::set_property(analyzer, dep, key, value);
    }
    let indeterminate = analyzer.is_assignment_indeterminate(self.cf_scope);
    let mut has_effect = false;
    if let Some(key_literals) = key.get_to_property_key().get_to_literals() {
      let definite = !indeterminate && key_literals.len() == 1;
      let mut rest_added = false;
      for key_literal in key_literals {
        match key_literal {
          LiteralEntity::String(key) => {
            if let Some(index) = key.parse::<usize>().ok() {
              if let Some(element) = self.elements.borrow_mut().get_mut(index) {
                *element = if definite {
                  value.clone()
                } else {
                  UnionEntity::new(vec![element.clone(), value.clone()])
                };
              } else if !rest_added {
                rest_added = true;
                self.rest.borrow_mut().push(value.clone());
              }
            } else if key == "length" {
              if let Some(length) = value.get_literal().and_then(|lit| lit.to_number()) {
                if let Some(length) = length.map(|l| l.0.to_js_int_32()) {
                  let length = length as usize;
                  let mut elements = self.elements.borrow_mut();
                  let mut rest = self.rest.borrow_mut();
                  if elements.len() > length {
                    has_effect = true;
                    elements.truncate(length);
                    rest.clear();
                  } else if !rest.is_empty() {
                    has_effect = true;
                    rest.push(LiteralEntity::new_undefined());
                  } else if elements.len() < length {
                    has_effect = true;
                    for _ in elements.len()..length {
                      elements.push(LiteralEntity::new_undefined());
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
              self.consume(analyzer);
              analyzer.consume(dep);
              return;
            }
          }
          LiteralEntity::Symbol(key, _) => todo!(),
          _ => unreachable!(),
        }
      }
      if has_effect {
        self.add_dep(analyzer, dep);
      }
    } else {
      self.consume(analyzer);
      consumed_object::set_property(analyzer, dep, key, value)
    }
  }

  fn enumerate_properties(
    &self,
    rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> Vec<(bool, Entity<'a>, Entity<'a>)> {
    if self.consumed.get() {
      return consumed_object::enumerate_properties(rc, analyzer, dep);
    }
    let self_dep = box_consumable((self.deps.borrow_mut().collect(), dep.cloned()));

    let mut entries = Vec::new();
    for (i, element) in self.elements.borrow().iter().enumerate() {
      entries.push((
        true,
        LiteralEntity::new_string(analyzer.allocator.alloc(i.to_string())),
        ForwardedEntity::new(element.clone(), self_dep.cloned()),
      ));
    }
    let rest = self.rest.borrow();
    if !rest.is_empty() {
      entries.push((
        true,
        UnknownEntity::new_string(),
        ForwardedEntity::new(UnionEntity::new(rest.iter().cloned().collect()), self_dep.cloned()),
      ));
    }
    entries
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, key: &Entity<'a>) {
    self.consume(analyzer);
    consumed_object::delete_property(analyzer, dep, key);
  }

  fn call(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: &Entity<'a>,
    args: &Entity<'a>,
  ) -> Entity<'a> {
    self.consume(analyzer);
    consumed_object::call(analyzer, dep, this, args)
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
    if self.consumed.get() {
      return consumed_object::iterate(analyzer, dep);
    }
    let rest = self.rest.borrow();
    (
      self
        .elements
        .borrow()
        .iter()
        .map(|val| ComputedEntity::new(val.clone(), dep.cloned()))
        .collect(),
      if rest.is_empty() {
        None
      } else {
        Some(UnionEntity::new_computed(self.rest.borrow().iter().cloned().collect(), dep))
      },
    )
  }

  fn get_typeof(&self) -> Entity<'a> {
    LiteralEntity::new_string("object")
  }

  fn get_to_string(&self, rc: &Entity<'a>) -> Entity<'a> {
    if self.consumed.get() {
      return consumed_object::get_to_string();
    }
    UnknownEntity::new_computed_string(rc.to_consumable())
  }

  fn get_to_numeric(&self, rc: &Entity<'a>) -> Entity<'a> {
    if self.consumed.get() {
      return consumed_object::get_to_numeric();
    }
    UnknownEntity::new_computed_unknown(rc.to_consumable())
  }

  fn get_to_boolean(&self, _rc: &Entity<'a>) -> Entity<'a> {
    LiteralEntity::new_boolean(true)
  }

  fn get_to_property_key(&self, rc: &Entity<'a>) -> Entity<'a> {
    self.get_to_string(rc)
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
  pub fn new(cf_scope: ScopeId, variable_scope: ScopeId) -> Self {
    ArrayEntity {
      consumed: Cell::new(false),
      deps: Default::default(),
      cf_scope,
      variable_scope,
      elements: RefCell::new(Vec::new()),
      rest: RefCell::new(Vec::new()),
    }
  }

  pub fn push_element(&self, element: Entity<'a>) {
    self.elements.borrow_mut().push(element);
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

  fn add_dep(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>) {
    let target_depth = analyzer.find_first_different_variable_scope(self.variable_scope);
    let mut deps = self.deps.borrow_mut();
    deps.push(box_consumable(analyzer.get_assignment_dep(target_depth)));
    deps.push(dep);
  }
}

impl<'a> Analyzer<'a> {
  pub fn new_empty_array(&self) -> ArrayEntity<'a> {
    ArrayEntity::new(self.scope_context.cf.current_id(), self.scope_context.variable.current_id())
  }
}
