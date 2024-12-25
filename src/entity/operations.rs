use super::{Entity, LiteralEntity, TypeofResult};
use crate::{analyzer::Analyzer, consumable::box_consumable, mangling::MangleConstraint};
use oxc::{
  allocator::Allocator,
  ast::ast::{BinaryOperator, UpdateOperator},
};

pub struct EntityOpHost<'a> {
  allocator: &'a Allocator,
}

impl<'a> EntityOpHost<'a> {
  pub fn new(allocator: &'a Allocator) -> Self {
    Self { allocator }
  }

  pub fn eq(
    &self,
    analyzer: &Analyzer<'a>,
    lhs: Entity<'a>,
    rhs: Entity<'a>,
  ) -> (Option<bool>, Option<MangleConstraint>) {
    if let (Some(true), m) = self.strict_eq(analyzer, lhs, rhs) {
      return (Some(true), m);
    }

    if lhs.test_nullish() == Some(true) && rhs.test_nullish() == Some(true) {
      return (Some(true), None);
    }

    (None, None)
  }

  pub fn neq(
    &self,
    analyzer: &Analyzer<'a>,
    lhs: Entity<'a>,
    rhs: Entity<'a>,
  ) -> (Option<bool>, Option<MangleConstraint>) {
    let (eq, m) = self.eq(analyzer, lhs, rhs);
    (eq.map(|v| !v), m.map(MangleConstraint::negate_equality))
  }

  pub fn strict_eq(
    &self,
    analyzer: &Analyzer<'a>,
    lhs: Entity<'a>,
    rhs: Entity<'a>,
  ) -> (Option<bool>, Option<MangleConstraint>) {
    // TODO: Find another way to do this
    // if Entity::ptr_eq(lhs, rhs) {
    //   return Some(true);
    // }

    let lhs_t = lhs.test_typeof();
    let rhs_t = rhs.test_typeof();
    if lhs_t & rhs_t == TypeofResult::_None {
      return (Some(false), None);
    }

    let lhs_lit = lhs.get_to_literals(analyzer);
    let rhs_lit = rhs.get_to_literals(analyzer);
    if let (Some(lhs_lit), Some(rhs_lit)) = (lhs_lit, rhs_lit) {
      if lhs_lit.len() == 1 && rhs_lit.len() == 1 {
        let lhs_lit = *lhs_lit.iter().next().unwrap();
        let rhs_lit = *rhs_lit.iter().next().unwrap();
        return lhs_lit.strict_eq(rhs_lit);
      }

      let mut constraints = Some(vec![]);
      let mut all_neq = true;
      'check: for l in &lhs_lit {
        for r in &rhs_lit {
          let (eq, mc) = l.strict_eq(*r);
          all_neq &= eq == Some(false);
          if let Some(mc) = mc {
            if let Some(constraints) = &mut constraints {
              constraints.push(mc);
            } else {
              constraints = None;
            }
          } else if !all_neq {
            break 'check;
          }
        }
      }

      return (
        if all_neq { Some(false) } else { None },
        constraints.map(MangleConstraint::Multiple),
      );
    }

    (None, None)
  }

  pub fn strict_neq(
    &self,
    analyzer: &Analyzer<'a>,
    lhs: Entity<'a>,
    rhs: Entity<'a>,
  ) -> (Option<bool>, Option<MangleConstraint>) {
    let (eq, m) = self.strict_eq(analyzer, lhs, rhs);
    (eq.map(|v| !v), m.map(MangleConstraint::negate_equality))
  }

  pub fn lt(
    &self,
    analyzer: &Analyzer<'a>,
    lhs: Entity<'a>,
    rhs: Entity<'a>,
    eq: bool,
  ) -> Option<bool> {
    fn literal_lt(lhs: &LiteralEntity, rhs: &LiteralEntity, eq: bool) -> Option<bool> {
      match (lhs, rhs) {
        (LiteralEntity::Number(l, _), LiteralEntity::Number(r, _)) => {
          Some(if eq { l.0 <= r.0 } else { l.0 < r.0 })
        }
        (LiteralEntity::String(l, _), LiteralEntity::String(r, _)) => {
          Some(if eq { l <= r } else { l < r })
        }
        (LiteralEntity::BigInt(_), LiteralEntity::BigInt(_))
        | (LiteralEntity::BigInt(_), LiteralEntity::String(_, _))
        | (LiteralEntity::String(_, _), LiteralEntity::BigInt(_)) => None,
        (lhs, rhs) => {
          let lhs = lhs.to_number();
          let rhs = rhs.to_number();
          match (lhs, rhs) {
            (None, _) | (_, None) => None,
            (Some(None), _) | (_, Some(None)) => Some(false),
            (Some(Some(l)), Some(Some(r))) => Some(if eq { l.0 <= r.0 } else { l.0 < r.0 }),
          }
        }
      }
    }

    if let (Some(lhs), Some(rhs)) = (lhs.get_to_literals(analyzer), rhs.get_to_literals(analyzer)) {
      let mut result = None;
      for lhs in lhs.iter() {
        for rhs in rhs.iter() {
          if let Some(v) = literal_lt(lhs, rhs, eq) {
            if let Some(result) = result {
              if result != v {
                return None;
              }
            } else {
              result = Some(v);
            }
          } else {
            return None;
          }
        }
      }
      debug_assert!(result.is_some());
      result
    } else {
      None
    }
  }

  pub fn gt(
    &self,
    analyzer: &Analyzer<'a>,
    lhs: Entity<'a>,
    rhs: Entity<'a>,
    eq: bool,
  ) -> Option<bool> {
    self.lt(analyzer, rhs, lhs, eq)
  }

  pub fn instanceof(&self, lhs: Entity<'a>, _rhs: Entity<'a>) -> Option<bool> {
    if (TypeofResult::String
      | TypeofResult::Number
      | TypeofResult::BigInt
      | TypeofResult::Boolean
      | TypeofResult::Symbol
      | TypeofResult::Undefined)
      .contains(lhs.test_typeof())
      || lhs.test_nullish() == Some(true)
    {
      Some(false)
    } else {
      None
    }
  }

  pub fn add(&self, analyzer: &Analyzer<'a>, lhs: Entity<'a>, rhs: Entity<'a>) -> Entity<'a> {
    let lhs_t = lhs.test_typeof();
    let rhs_t = rhs.test_typeof();
    let lhs_lit = lhs.get_literal(analyzer);
    let rhs_lit = rhs.get_literal(analyzer);

    let mut values = vec![];

    let may_convert_to_num = TypeofResult::Number
      | TypeofResult::Boolean
      | TypeofResult::Undefined
      | TypeofResult::Object
      | TypeofResult::Function;
    let must_not_convert_to_str =
      TypeofResult::Number | TypeofResult::Boolean | TypeofResult::Undefined | TypeofResult::BigInt;

    if lhs_t.intersects(may_convert_to_num) && rhs_t.intersects(may_convert_to_num) {
      // Possibly number
      match (lhs_lit.and_then(|v| v.to_number()), rhs_lit.and_then(|v| v.to_number())) {
        (Some(l), Some(r)) => match (l, r) {
          (Some(l), Some(r)) => {
            let val = l.0 + r.0;
            values.push(analyzer.factory.number(val, None));
          }
          _ => {
            values.push(analyzer.factory.nan);
          }
        },
        _ => {
          values.push(analyzer.factory.unknown_number);
        }
      }
    }
    if lhs_t.contains(TypeofResult::BigInt) && rhs_t.contains(TypeofResult::BigInt) {
      // Possibly bigint
      values.push(analyzer.factory.unknown_bigint);
    }
    if !lhs_t.difference(must_not_convert_to_str).is_empty()
      || !rhs_t.difference(must_not_convert_to_str).is_empty()
    {
      let lhs_str = lhs.get_to_string(analyzer);
      let rhs_str = rhs.get_to_string(analyzer);

      let lhs_str_lit = lhs_str.get_literal(analyzer);
      let rhs_str_lit = rhs_str.get_literal(analyzer);

      match (lhs_str_lit, rhs_str_lit) {
        (Some(LiteralEntity::String(l, _)), Some(LiteralEntity::String(r, _))) => {
          let val = l.to_string() + r;
          values.push(analyzer.factory.string(self.allocator.alloc(val)));
        }
        _ => {
          values.push(analyzer.factory.unknown_string);
        }
      }
    }

    let dep = box_consumable((lhs, rhs));
    if values.is_empty() {
      // TODO: throw warning
      analyzer.factory.computed_unknown(dep)
    } else {
      analyzer.factory.computed_union(values, dep)
    }
  }

  fn number_only_op(
    &self,
    analyzer: &Analyzer<'a>,
    lhs: Entity<'a>,
    rhs: Entity<'a>,
    calc: fn(f64, f64) -> f64,
  ) -> Entity<'a> {
    analyzer.factory.computed(
      if let (Some(LiteralEntity::Number(lhs, _)), Some(LiteralEntity::Number(rhs, _))) =
        (lhs.get_literal(analyzer), rhs.get_literal(analyzer))
      {
        analyzer.factory.number(calc(lhs.0, rhs.0), None)
      } else {
        analyzer.factory.unknown_primitive
      },
      (lhs, rhs),
    )
  }

  pub fn update(
    &self,
    analyzer: &Analyzer<'a>,
    input: Entity<'a>,
    operator: UpdateOperator,
  ) -> Entity<'a> {
    let apply_update = |v: f64| {
      let val = match operator {
        UpdateOperator::Increment => v + 1.0,
        UpdateOperator::Decrement => v - 1.0,
      };
      analyzer.factory.number(val, None)
    };

    if let Some(num) = input.get_literal(analyzer).and_then(|lit| lit.to_number()) {
      return analyzer.factory.computed(
        match num {
          Some(num) => apply_update(num.0),
          None => analyzer.factory.nan,
        },
        input,
      );
    }

    let input_t = input.test_typeof();

    let mut values = vec![];
    if input_t.contains(TypeofResult::BigInt) {
      values.push(analyzer.factory.unknown_bigint);
    }
    if input_t.contains(TypeofResult::Number) {
      values.push(analyzer.factory.unknown_number);
    }

    if values.is_empty() {
      analyzer.factory.computed_unknown(input)
    } else {
      analyzer.factory.computed_union(values, input)
    }
  }

  pub fn binary_op(
    &self,
    analyzer: &Analyzer<'a>,
    operator: BinaryOperator,
    lhs: Entity<'a>,
    rhs: Entity<'a>,
  ) -> Entity<'a> {
    let to_result = |result: Option<bool>| {
      analyzer.factory.computed(analyzer.factory.boolean_maybe_unknown(result), (lhs, rhs))
    };

    let to_eq_result = |(equality, mangle_constraint): (Option<bool>, Option<MangleConstraint>)| {
      if let Some(mangle_constraint) = mangle_constraint {
        analyzer.factory.mangable(
          analyzer.factory.boolean_maybe_unknown(equality),
          (lhs, rhs),
          analyzer.factory.alloc(mangle_constraint),
        )
      } else {
        to_result(equality)
      }
    };

    match operator {
      BinaryOperator::Equality => to_eq_result(self.eq(analyzer, lhs, rhs)),
      BinaryOperator::Inequality => to_eq_result(self.neq(analyzer, lhs, rhs)),
      BinaryOperator::StrictEquality => to_eq_result(self.strict_eq(analyzer, lhs, rhs)),
      BinaryOperator::StrictInequality => to_eq_result(self.strict_neq(analyzer, lhs, rhs)),
      BinaryOperator::LessThan => to_result(self.lt(analyzer, lhs, rhs, false)),
      BinaryOperator::LessEqualThan => to_result(self.lt(analyzer, lhs, rhs, true)),
      BinaryOperator::GreaterThan => to_result(self.gt(analyzer, lhs, rhs, false)),
      BinaryOperator::GreaterEqualThan => to_result(self.gt(analyzer, lhs, rhs, true)),
      BinaryOperator::Addition => self.add(analyzer, lhs, rhs),
      BinaryOperator::Subtraction => self.number_only_op(analyzer, lhs, rhs, |l, r| l - r),
      BinaryOperator::Multiplication => self.number_only_op(analyzer, lhs, rhs, |l, r| l * r),
      BinaryOperator::ShiftLeft => {
        self.number_only_op(analyzer, lhs, rhs, |l, r| l.floor() * 2f64.powf(r))
      }
      BinaryOperator::ShiftRight => {
        self.number_only_op(analyzer, lhs, rhs, |l, r| l.floor() / 2f64.powf(r))
      }
      BinaryOperator::ShiftRightZeroFill => {
        self.number_only_op(analyzer, lhs, rhs, |l, r| l.floor() / 2f64.powf(r))
      }
      BinaryOperator::Division => self.number_only_op(analyzer, lhs, rhs, |l, r| l / r),
      BinaryOperator::Remainder => self.number_only_op(analyzer, lhs, rhs, |l, r| l % r),
      BinaryOperator::BitwiseOR => {
        self.number_only_op(analyzer, lhs, rhs, |l, r| (l as i64 | r as i64) as f64)
      }
      BinaryOperator::BitwiseXOR => {
        self.number_only_op(analyzer, lhs, rhs, |l, r| (l as i64 ^ r as i64) as f64)
      }
      BinaryOperator::BitwiseAnd => {
        self.number_only_op(analyzer, lhs, rhs, |l, r| (l as i64 & r as i64) as f64)
      }
      BinaryOperator::Exponential => self.number_only_op(analyzer, lhs, rhs, |l, r| l.powf(r)),
      BinaryOperator::In => analyzer.factory.computed_unknown_boolean((lhs, rhs)),
      BinaryOperator::Instanceof => to_result(self.instanceof(lhs, rhs)),
    }
  }
}
