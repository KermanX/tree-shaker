use rustc_hash::FxHashSet;

use crate::utils::is_numeric_string;

use super::EntityValue;
use std::{ops::Deref, rc::Rc};

impl EntityValue {
  pub fn simplify(&self) -> EntityValue {
    match self {
      EntityValue::Union(values) => Self::simplify_union(values),
      other => other.clone(),
    }
  }

  pub(crate) fn simplify_union(values: &Vec<Rc<EntityValue>>) -> EntityValue {
    let values: Vec<Rc<EntityValue>> = values
      .iter()
      .flat_map(|e| {
        let entity = e.simplify();
        match entity {
          EntityValue::Union(values) => values,
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
        EntityValue::UnknownString => has_unknown_string = true,
        EntityValue::UnknownNumber => has_unknown_number = true,
        EntityValue::UnknownBigInt => has_unknown_bigint = true,
        EntityValue::UnknownSymbol => has_unknown_symbol = true,
        EntityValue::UnknownFunction => has_unknown_function = true,
        EntityValue::Unknown => return EntityValue::Unknown,
        _ => {}
      }
    }

    let mut has_numeric_string = false;
    let mut has_non_empty_string = false;
    let mut has_non_zero_number = false;
    let mut has_non_zero_bigint = false;

    for value in values.iter() {
      match value.deref() {
        EntityValue::NonEmptyString(true) if !has_unknown_string => has_numeric_string = true,
        EntityValue::NonEmptyString(false) if !has_unknown_string => has_non_empty_string = true,
        EntityValue::NonZeroNumber if !has_unknown_number => has_non_zero_number = true,
        EntityValue::NonZeroBigInt if !has_unknown_bigint => has_non_zero_bigint = true,
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
    let mut has_others = Vec::<Rc<EntityValue>>::default();

    for value in values.iter() {
      match value.deref() {
        EntityValue::StringLiteral(str) if !has_unknown_string => {
          if !has_non_empty_string {
            if !has_numeric_string || !is_numeric_string(str.as_str()) {
              has_string_literal.insert(str.clone());
            }
          } else if str.is_empty() {
            has_unknown_string = true;
            has_non_empty_string = false;
          }
        }
        EntityValue::NumberLiteral(num) if !has_unknown_number => {
          if !has_non_zero_number {
            if !has_number_literal.contains(num) {
              has_number_literal.push(*num);
            }
          } else if *num == 0.0 {
            has_unknown_number = true;
            has_non_zero_number = false;
          }
        }
        EntityValue::BigIntLiteral(num) if !has_unknown_bigint => {
          if !has_non_zero_bigint {
            has_bigint_literal.insert(*num);
          } else if *num == 0 {
            has_unknown_bigint = true;
            has_non_zero_bigint = false;
          }
        }
        EntityValue::BooleanLiteral(true) => has_boolean_true = true,
        EntityValue::BooleanLiteral(false) => has_boolean_false = true,
        EntityValue::Null => {
          has_null = true;
        }
        EntityValue::Undefined => {
          has_undefined = true;
        }
        EntityValue::Symbol(_) if !has_unknown_symbol => {
          // TODO: Handle same symbol
          has_others.push(value.clone());
        }
        EntityValue::Function(_) if !has_unknown_function => {
          // TODO: Handle same function
          has_others.push(value.clone());
        }
        EntityValue::Array(_) | EntityValue::Object(_) => {
          has_others.push(value.clone());
        }
        _ => {}
      }
    }

    let mut result: Vec<Rc<EntityValue>> = Vec::new();

    if has_unknown_string {
      result.push(Rc::new(EntityValue::UnknownString));
    }
    if has_numeric_string {
      result.push(Rc::new(EntityValue::NonEmptyString(true)));
    }
    if has_non_empty_string {
      result.push(Rc::new(EntityValue::NonEmptyString(false)));
    }
    for str in has_string_literal {
      result.push(Rc::new(EntityValue::StringLiteral(str)));
    }
    if has_unknown_number {
      result.push(Rc::new(EntityValue::UnknownNumber));
    }
    if has_non_zero_number {
      result.push(Rc::new(EntityValue::NonZeroNumber));
    }
    for num in has_number_literal {
      result.push(Rc::new(EntityValue::NumberLiteral(num)));
    }
    if has_unknown_bigint {
      result.push(Rc::new(EntityValue::UnknownBigInt));
    }
    if has_non_zero_bigint {
      result.push(Rc::new(EntityValue::NonZeroBigInt));
    }
    for num in has_bigint_literal {
      result.push(Rc::new(EntityValue::BigIntLiteral(num)));
    }
    if has_boolean_true {
      result.push(Rc::new(EntityValue::BooleanLiteral(true)));
    }
    if has_boolean_false {
      result.push(Rc::new(EntityValue::BooleanLiteral(false)));
    }
    if has_null {
      result.push(Rc::new(EntityValue::Null));
    }
    if has_undefined {
      result.push(Rc::new(EntityValue::Undefined));
    }
    if has_unknown_symbol {
      result.push(Rc::new(EntityValue::UnknownSymbol));
    }
    for value in has_others {
      result.push(value);
    }

    if result.len() == 1 {
      result[0].deref().clone()
    } else {
      EntityValue::Union(result)
    }
  }
}
