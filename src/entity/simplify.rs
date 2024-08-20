use rustc_hash::FxHashSet;

use crate::utils::is_numeric_string;

use super::Entity;
use std::{ops::Deref, rc::Rc};

impl Entity {
  pub fn simplify(&self) -> Entity {
    match self {
      Entity::Union(values) => Self::simplify_union(values),
      other => other.clone(),
    }
  }

  pub(crate) fn simplify_union(values: &Vec<Rc<Entity>>) -> Entity {
    let mut values: Vec<Rc<Entity>> = values
      .iter()
      .flat_map(|e| {
        let entity = e.simplify();
        match entity {
          Entity::Union(values) => values,
          _ => vec![Rc::new(entity.clone())],
        }
      })
      .collect();

    let mut has_unknown_string = false;
    let mut has_unknown_number = false;
    let mut has_unknown_bigint = false;
    let mut has_unknown_symbol = false;
    let mut has_unknown_function = false;

    for value in values.iter() {
      match value.deref() {
        Entity::UnknownString => has_unknown_string = true,
        Entity::UnknownNumber => has_unknown_number = true,
        Entity::UnknownBigInt => has_unknown_bigint = true,
        Entity::UnknownSymbol => has_unknown_symbol = true,
        Entity::UnknownFunction => has_unknown_function = true,
        Entity::Unknown => return Entity::Unknown,
        _ => {}
      }
    }

    let mut has_numeric_string = false;
    let mut has_non_empty_string = false;
    let mut has_non_zero_number = false;
    let mut has_non_zero_bigint = false;

    for value in values.iter() {
      match value.deref() {
        Entity::NonEmptyString(true) if !has_unknown_string => has_numeric_string = true,
        Entity::NonEmptyString(false) if !has_unknown_string => has_non_empty_string = true,
        Entity::NonZeroNumber if !has_unknown_number => has_non_zero_number = true,
        Entity::NonZeroBigInt if !has_unknown_bigint => has_non_zero_bigint = true,
        _ => {}
      }
    }

    if has_non_empty_string {
      has_numeric_string = false;
    }

    let mut has_string_literal = FxHashSet::<String>::default();
    let mut has_number_literal = Vec::<f64>::default();
    let mut has_bigint_literal = FxHashSet::<i64>::default();
    let mut has_boolean_true = false;
    let mut has_boolean_false = false;
    let mut has_null = false;
    let mut has_undefined = false;
    let mut has_others = Vec::<Rc<Entity>>::default();

    for value in values.iter() {
      match value.deref() {
        Entity::StringLiteral(str) if !has_unknown_string => {
          if !has_non_empty_string {
            if !has_numeric_string || !is_numeric_string(str.as_str()) {
              has_string_literal.insert(str.clone());
            }
          } else if str.is_empty() {
            has_unknown_string = true;
            has_non_empty_string = false;
          }
        }
        Entity::NumberLiteral(num) if !has_unknown_number => {
          if !has_non_zero_number {
            if !has_number_literal.contains(num) {
              has_number_literal.push(*num);
            }
          } else if *num == 0.0 {
            has_unknown_number = true;
            has_non_zero_number = false;
          }
        }
        Entity::BigIntLiteral(num) if !has_unknown_bigint => {
          if !has_non_zero_bigint {
            has_bigint_literal.insert(*num);
          } else if *num == 0 {
            has_unknown_bigint = true;
            has_non_zero_bigint = false;
          }
        }
        Entity::BooleanLiteral(true) => has_boolean_true = true,
        Entity::BooleanLiteral(false) => has_boolean_false = true,
        Entity::Null => {
          has_null = true;
        }
        Entity::Undefined => {
          has_undefined = true;
        }
        Entity::Symbol(_) if !has_unknown_symbol => {
          // TODO: Handle same symbol
          has_others.push(value.clone());
        }
        Entity::Function(_) if !has_unknown_function => {
          // TODO: Handle same function
          has_others.push(value.clone());
        }
        Entity::Array(_) | Entity::Object(_) => {
          has_others.push(value.clone());
        }
        _ => {}
      }
    }

    let mut result: Vec<Rc<Entity>> = Vec::new();

    if has_unknown_string {
      result.push(Rc::new(Entity::UnknownString));
    }
    if has_numeric_string {
      result.push(Rc::new(Entity::NonEmptyString(true)));
    }
    if has_non_empty_string {
      result.push(Rc::new(Entity::NonEmptyString(false)));
    }
    for str in has_string_literal {
      result.push(Rc::new(Entity::StringLiteral(str)));
    }
    if has_unknown_number {
      result.push(Rc::new(Entity::UnknownNumber));
    }
    if has_non_zero_number {
      result.push(Rc::new(Entity::NonZeroNumber));
    }
    for num in has_number_literal {
      result.push(Rc::new(Entity::NumberLiteral(num)));
    }
    if has_unknown_bigint {
      result.push(Rc::new(Entity::UnknownBigInt));
    }
    if has_non_zero_bigint {
      result.push(Rc::new(Entity::NonZeroBigInt));
    }
    for num in has_bigint_literal {
      result.push(Rc::new(Entity::BigIntLiteral(num)));
    }
    if has_boolean_true {
      result.push(Rc::new(Entity::BooleanLiteral(true)));
    }
    if has_boolean_false {
      result.push(Rc::new(Entity::BooleanLiteral(false)));
    }
    if has_null {
      result.push(Rc::new(Entity::Null));
    }
    if has_undefined {
      result.push(Rc::new(Entity::Undefined));
    }
    if has_unknown_symbol {
      result.push(Rc::new(Entity::UnknownSymbol));
    }
    for value in has_others {
      result.push(value);
    }

    if result.len() == 1 {
      result[0].deref().clone()
    } else {
      Entity::Union(result)
    }
  }
}
