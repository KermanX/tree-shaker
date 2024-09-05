use std::rc::Rc;
use crate::entity::union::UnionEntity;
use super::{
  entity::Entity,
  literal::LiteralEntity,
  typeof_result::TypeofResult,
  unknown::{UnknownEntity, UnknownEntityKind},
};
use oxc::allocator::Allocator;

pub struct EntityOpHost<'a> {
  allocator: &'a Allocator,
}

impl<'a> EntityOpHost<'a> {
  pub fn new(allocator: &'a Allocator) -> Self {
    Self { allocator }
  }

  pub fn strict_eq(&self, lhs: &Entity<'a>, rhs: &Entity<'a>) -> Option<bool> {
    if Rc::ptr_eq(lhs, rhs) {
      return Some(true);
    }

    let lhs_t = lhs.test_typeof();
    let rhs_t = rhs.test_typeof();
    if lhs_t & rhs_t == TypeofResult::_None {
      return Some(false);
    }

    let lhs_lit = lhs.get_to_literals();
    let rhs_lit = rhs.get_to_literals();
    if let (Some(lhs_lit), Some(rhs_lit)) = (lhs_lit, rhs_lit) {
      if lhs_lit.len() == 1 && rhs_lit.len() == 1 {
        let lhs_lit = lhs_lit.iter().next().unwrap();
        let rhs_lit = rhs_lit.iter().next().unwrap();
        return Some(lhs_lit == rhs_lit);
      }

      if lhs_lit.iter().all(|lit| !rhs_lit.contains(lit)) {
        return Some(false);
      }
    }

    None
  }

  pub fn add(&self, lhs: &Entity<'a>, rhs: &Entity<'a>) -> Entity<'a> {
    let lhs_t = lhs.test_typeof();
    let rhs_t = rhs.test_typeof();
    let lhs_lit = lhs.get_literal();
    let rhs_lit = rhs.get_literal();

    let mut values = vec![];

    if lhs_t.contains(TypeofResult::Number) && rhs_t.contains(TypeofResult::Number) {
      // Possibly number
      match (lhs_lit, rhs_lit) {
        (Some(LiteralEntity::Number(l, _)), Some(LiteralEntity::Number(r, _))) => {
          let val = l.0 + r.0;
          values.push(LiteralEntity::new_number(val.into(), self.allocator.alloc(val.to_string())));
        }
        _ => {
          values.push(UnknownEntity::new_with_deps(
            UnknownEntityKind::Number,
            vec![lhs.clone(), rhs.clone()],
          ));
        }
      }
    }
    if lhs_t.contains(TypeofResult::BigInt) && rhs_t.contains(TypeofResult::BigInt) {
      // Possibly bigint
      values.push(UnknownEntity::new_with_deps(
        UnknownEntityKind::BigInt,
        vec![lhs.clone(), rhs.clone()],
      ));
    }
    if !lhs_t.difference(TypeofResult::Number | TypeofResult::BigInt).is_empty()
      || !rhs_t.difference(TypeofResult::Number | TypeofResult::BigInt).is_empty()
    {
      let lhs_str = lhs.get_to_string();
      let rhs_str = rhs.get_to_string();

      let lhs_str_lit = lhs_str.get_literal();
      let rhs_str_lit = rhs_str.get_literal();

      match (lhs_str_lit, rhs_str_lit) {
        (Some(LiteralEntity::String(l)), Some(LiteralEntity::String(r))) => {
          let val = l.to_string() + r;
          values.push(LiteralEntity::new_string(self.allocator.alloc(val)));
        }
        _ => {
          values
            .push(UnknownEntity::new_with_deps(UnknownEntityKind::String, vec![lhs_str, rhs_str]));
        }
      }
    }

    debug_assert!(values.len() > 0);

    UnionEntity::new(values)
  }
}
