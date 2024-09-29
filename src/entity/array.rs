use super::{
  consumed_object, ComputedEntity, Consumable, Entity, EntityTrait, ForwardedEntity,
  InteractionKind, LiteralEntity, TypeofResult, UnionEntity, UnknownEntity,
};
use crate::{
  analyzer::Analyzer,
  scope::{cf_scope::CfScopes, variable_scope::VariableScopes},
  use_consumed_flag,
};
use oxc::syntax::number::ToJsInt32;
use std::{
  cell::{Cell, RefCell},
  fmt, mem,
};

pub struct ArrayEntity<'a> {
  consumed: Cell<bool>,
  pub deps: RefCell<Vec<Consumable<'a>>>,
  cf_scopes: CfScopes<'a>,
  variable_scopes: VariableScopes<'a>,
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

    analyzer.consume(mem::take(&mut *self.deps.borrow_mut()));

    for element in self.elements.borrow().iter() {
      element.consume(analyzer);
    }

    for rest in self.rest.borrow().iter() {
      rest.consume(analyzer);
    }
  }

  fn interact(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, kind: InteractionKind) {
    self.consume(analyzer);
    consumed_object::interact(analyzer, dep, kind);
  }

  fn get_property(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: &Entity<'a>,
  ) -> Entity<'a> {
    if self.consumed.get() {
      return consumed_object::get_property(analyzer, dep, key);
    }
    let mut deps = self.deps.borrow().clone();
    deps.push(dep);
    deps.push(key.clone().into());
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
                  UnknownEntity::new_computed_number(dep)
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
      UnionEntity::new_computed(result, deps)
    } else {
      UnknownEntity::new_computed_unknown(
        self
          .elements
          .borrow()
          .iter()
          .chain(self.rest.borrow().iter())
          .map(|v| Consumable::from(v.clone()))
          .chain(deps)
          .collect::<Vec<_>>(),
      )
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
    let indeterminate = analyzer.is_assignment_indeterminate(&self.cf_scopes);
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
                  // TODO: throw warning: Invalid array length
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
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> Vec<(bool, Entity<'a>, Entity<'a>)> {
    if self.consumed.get() {
      return consumed_object::enumerate_properties(analyzer, dep);
    }
    let mut deps = self.deps.borrow().clone();
    deps.push(dep.clone());
    let self_dep = Consumable::new(deps);

    let mut entries = Vec::new();
    for (i, element) in self.elements.borrow().iter().enumerate() {
      entries.push((
        true,
        LiteralEntity::new_string(analyzer.allocator.alloc(i.to_string())),
        ForwardedEntity::new(element.clone(), self_dep.clone()),
      ));
    }
    let rest = self.rest.borrow();
    if !rest.is_empty() {
      entries.push((
        true,
        UnknownEntity::new_string(),
        ForwardedEntity::new(UnionEntity::new(rest.iter().cloned().collect()), self_dep.clone()),
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
        .map(|val| ComputedEntity::new(val.clone(), dep.clone()))
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
    UnknownEntity::new_computed_string(rc.clone())
  }

  fn get_to_numeric(&self, rc: &Entity<'a>) -> Entity<'a> {
    if self.consumed.get() {
      return consumed_object::get_to_numeric();
    }
    UnknownEntity::new_computed_unknown(rc.clone())
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
  pub fn new(cf_scopes: CfScopes<'a>, variable_scopes: VariableScopes<'a>) -> Self {
    ArrayEntity {
      consumed: Cell::new(false),
      deps: RefCell::new(Vec::new()),
      cf_scopes,
      variable_scopes,
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

  fn add_dep(&self, analyzer: &Analyzer<'a>, dep: Consumable<'a>) {
    let target_variable_scope = analyzer.find_first_different_variable_scope(&self.variable_scopes);
    self.deps.borrow_mut().push(analyzer.get_assignment_deps(target_variable_scope, dep));
  }
}

impl<'a> Analyzer<'a> {
  pub fn new_empty_array(&self) -> ArrayEntity<'a> {
    ArrayEntity::new(
      self.scope_context.cf_scopes.clone(),
      self.scope_context.variable_scopes.clone(),
    )
  }
}
