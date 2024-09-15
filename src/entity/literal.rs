use super::{
  consumed_object,
  dep::EntityDep,
  entity::{Entity, EntityTrait},
  typeof_result::TypeofResult,
  unknown::{UnknownEntity, UnknownEntityKind},
};
use crate::{analyzer::Analyzer, builtins::Prototype, utils::F64WithEq};
use oxc::{
  ast::{
    ast::{BigintBase, Expression, NumberBase, UnaryOperator},
    AstBuilder,
  },
  semantic::SymbolId,
  span::Span,
};
use rustc_hash::FxHashSet;
use std::hash::{Hash, Hasher};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LiteralEntity<'a> {
  String(&'a str),
  Number(F64WithEq, &'a str),
  BigInt(&'a str),
  Boolean(bool),
  Symbol(SymbolId, &'a str),
  Infinity(bool),
  NaN,
  Null,
  Undefined,
}

impl<'a> EntityTrait<'a> for LiteralEntity<'a> {
  fn consume_self(&self, _analyzer: &mut Analyzer<'a>) {}
  fn consume_as_unknown(&self, _analyzer: &mut Analyzer<'a>) {}

  fn get_property(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
    key: &Entity<'a>,
  ) -> Entity<'a> {
    if matches!(self, LiteralEntity::Null | LiteralEntity::Undefined) {
      // TODO: throw warning
      consumed_object::get_property(analyzer, dep, key)
    } else {
      let prototype = self.get_prototype(analyzer);
      prototype.get_property(key, dep)
    }
  }

  fn set_property(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
    key: &Entity<'a>,
    value: Entity<'a>,
  ) {
    if matches!(self, LiteralEntity::Null | LiteralEntity::Undefined) {
      // TODO: throw warning
      consumed_object::set_property(analyzer, dep, key, value)
    } else {
      // No effect
    }
  }

  fn enumerate_properties(
    &self,
    _rc: &Entity<'a>,
    _analyzer: &mut Analyzer<'a>,
    _dep: EntityDep,
  ) -> Vec<(bool, Entity<'a>, Entity<'a>)> {
    // No effect
    vec![]
  }

  fn delete_property(&self, _analyzer: &mut Analyzer<'a>, _key: &Entity<'a>) -> bool {
    if matches!(self, LiteralEntity::Null | LiteralEntity::Undefined) {
      // TODO: throw warning
      true
    } else {
      // No effect
      false
    }
  }

  fn call(
    &self,
    _rc: &Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: EntityDep,
    this: &Entity<'a>,
    args: &Entity<'a>,
  ) -> Entity<'a> {
    // TODO: throw warning
    consumed_object::call(analyzer, dep, this, args)
  }

  fn r#await(&self, rc: &Entity<'a>, _analyzer: &mut Analyzer<'a>) -> (bool, Entity<'a>) {
    (false, rc.clone())
  }

  fn iterate(&self, rc: &Entity<'a>, _analyzer: &mut Analyzer<'a>) -> (bool, Option<Entity<'a>>) {
    match self {
      LiteralEntity::String(value) => (
        false,
        if value.is_empty() {
          None
        } else {
          Some(UnknownEntity::new_with_deps(UnknownEntityKind::String, vec![rc.clone()]))
        },
      ),
      _ => {
        // TODO: throw warning
        (true, Some(UnknownEntity::new_unknown()))
      }
    }
  }

  fn get_typeof(&self) -> Entity<'a> {
    LiteralEntity::new_string(self.test_typeof().to_string().unwrap())
  }

  fn get_to_string(&self, _rc: &Entity<'a>) -> Entity<'a> {
    LiteralEntity::new_string(self.to_string())
  }

  fn get_to_property_key(&self, rc: &Entity<'a>) -> Entity<'a> {
    match self {
      LiteralEntity::Symbol(_, _) => Entity::new(*self),
      _ => self.get_to_string(rc),
    }
  }

  fn get_to_array(&self, _rc: &Entity<'a>, length: usize) -> (Vec<Entity<'a>>, Entity<'a>) {
    UnknownEntity::new_unknown_to_array_result(length, vec![])
  }

  fn get_to_literals(&self) -> Option<FxHashSet<LiteralEntity<'a>>> {
    let mut result = FxHashSet::default();
    result.insert(*self);
    Some(result)
  }

  fn get_literal(&self) -> Option<LiteralEntity<'a>> {
    Some(*self)
  }

  fn test_typeof(&self) -> TypeofResult {
    match self {
      LiteralEntity::String(_) => TypeofResult::String,
      LiteralEntity::Number(_, _) => TypeofResult::Number,
      LiteralEntity::BigInt(_) => TypeofResult::BigInt,
      LiteralEntity::Boolean(_) => TypeofResult::Boolean,
      LiteralEntity::Symbol(_, _) => TypeofResult::Symbol,
      LiteralEntity::Infinity(_) => TypeofResult::Number,
      LiteralEntity::NaN => TypeofResult::Number,
      LiteralEntity::Null => TypeofResult::Object,
      LiteralEntity::Undefined => TypeofResult::Undefined,
    }
  }

  fn test_truthy(&self) -> Option<bool> {
    Some(match self {
      LiteralEntity::String(value) => !value.is_empty(),
      LiteralEntity::Number(value, _) => *value != 0.0.into(),
      LiteralEntity::BigInt(value) => !value.is_empty(),
      LiteralEntity::Boolean(value) => *value,
      LiteralEntity::Symbol(_, _) => true,
      LiteralEntity::Infinity(_) => true,
      LiteralEntity::NaN | LiteralEntity::Null | LiteralEntity::Undefined => false,
    })
  }

  fn test_nullish(&self) -> Option<bool> {
    Some(matches!(self, LiteralEntity::Null | LiteralEntity::Undefined))
  }
}

impl<'a> Hash for LiteralEntity<'a> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    match self {
      LiteralEntity::String(value) => {
        state.write_u8(0);
        value.hash(state);
      }
      LiteralEntity::Number(_, raw) => {
        state.write_u8(1);
        raw.hash(state);
      }
      LiteralEntity::BigInt(value) => {
        state.write_u8(2);
        value.hash(state);
      }
      LiteralEntity::Boolean(value) => {
        state.write_u8(3);
        value.hash(state);
      }
      LiteralEntity::Symbol(value, _) => {
        state.write_u8(4);
        value.hash(state);
      }
      LiteralEntity::Infinity(positive) => {
        state.write_u8(5);
        positive.hash(state);
      }
      LiteralEntity::NaN => {
        state.write_u8(6);
      }
      LiteralEntity::Null => {
        state.write_u8(7);
      }
      LiteralEntity::Undefined => {
        state.write_u8(8);
      }
    }
  }
}

impl<'a> LiteralEntity<'a> {
  pub fn new_string(value: &'a str) -> Entity<'a> {
    Entity::new(LiteralEntity::String(value))
  }

  pub fn new_number(value: F64WithEq, raw: &'a str) -> Entity<'a> {
    Entity::new(LiteralEntity::Number(value, raw))
  }

  pub fn new_big_int(value: &'a str) -> Entity<'a> {
    Entity::new(LiteralEntity::BigInt(value))
  }

  pub fn new_boolean(value: bool) -> Entity<'a> {
    Entity::new(LiteralEntity::Boolean(value))
  }

  pub fn new_infinity(positive: bool) -> Entity<'a> {
    Entity::new(LiteralEntity::Infinity(positive))
  }

  pub fn new_nan() -> Entity<'a> {
    Entity::new(LiteralEntity::NaN)
  }

  pub fn new_null() -> Entity<'a> {
    Entity::new(LiteralEntity::Null)
  }

  pub fn new_undefined() -> Entity<'a> {
    Entity::new(LiteralEntity::Undefined)
  }

  pub fn new_symbol(id: SymbolId, str_rep: &'a str) -> Entity<'a> {
    Entity::new(LiteralEntity::Symbol(id, str_rep))
  }

  pub fn build_expr(&self, ast_builder: &AstBuilder<'a>, span: Span) -> Expression<'a> {
    match self {
      LiteralEntity::String(value) => ast_builder.expression_string_literal(span, *value),
      LiteralEntity::Number(value, raw) => {
        ast_builder.expression_numeric_literal(span, value.0, *raw, NumberBase::Decimal)
      }
      LiteralEntity::BigInt(value) => {
        ast_builder.expression_big_int_literal(span, *value, BigintBase::Decimal)
      }
      LiteralEntity::Boolean(value) => ast_builder.expression_boolean_literal(span, *value),
      LiteralEntity::Symbol(_, _) => unreachable!(),
      LiteralEntity::Infinity(positive) => {
        if *positive {
          ast_builder.expression_identifier_reference(span, "Infinity")
        } else {
          ast_builder.expression_unary(
            span,
            UnaryOperator::UnaryNegation,
            ast_builder.expression_identifier_reference(span, "Infinity"),
          )
        }
      }
      LiteralEntity::NaN => ast_builder.expression_identifier_reference(span, "NaN"),
      LiteralEntity::Null => ast_builder.expression_null_literal(span),
      LiteralEntity::Undefined => ast_builder.expression_identifier_reference(span, "undefined"),
    }
  }

  pub fn can_build_expr(&self, analyzer: &Analyzer<'a>) -> bool {
    let config = &analyzer.config;
    match self {
      LiteralEntity::String(value) => value.len() <= config.max_simple_string_length,
      LiteralEntity::Number(value, _) => {
        value.0.fract() == 0.0
          && config.min_simple_number_value <= (value.0 as i64)
          && (value.0 as i64) <= config.max_simple_number_value
      }
      LiteralEntity::BigInt(_) => false,
      LiteralEntity::Boolean(_) => true,
      LiteralEntity::Symbol(_, _) => false,
      LiteralEntity::Infinity(_) => true,
      LiteralEntity::NaN => true,
      LiteralEntity::Null => true,
      LiteralEntity::Undefined => true,
    }
  }

  pub fn to_string(&self) -> &'a str {
    match self {
      LiteralEntity::String(value) => *value,
      LiteralEntity::Number(_, raw) => *raw,
      LiteralEntity::BigInt(value) => *value,
      LiteralEntity::Boolean(value) => {
        if *value {
          "true"
        } else {
          "false"
        }
      }
      LiteralEntity::Symbol(_, str_rep) => str_rep,
      LiteralEntity::Infinity(positive) => {
        if *positive {
          "Infinity"
        } else {
          "-Infinity"
        }
      }
      LiteralEntity::NaN => "NaN",
      LiteralEntity::Null => "null",
      LiteralEntity::Undefined => "undefined",
    }
  }

  // `None` for unresolvable, `Some(None)` for NaN, `Some(Some(value))` for number
  pub fn to_number(&self) -> Option<Option<F64WithEq>> {
    match self {
      LiteralEntity::Number(value, _) => Some(Some(*value)),
      LiteralEntity::BigInt(_value) => {
        // TODO: warn: TypeError: Cannot convert a BigInt value to a number
        None
      }
      LiteralEntity::Boolean(value) => Some(Some(if *value { 1.0 } else { 0.0 }.into())),
      LiteralEntity::String(value) => {
        let value = value.parse::<f64>();
        if let Ok(value) = value {
          Some(Some(value.into()))
        } else {
          Some(None)
        }
      }
      LiteralEntity::Null => Some(Some(0.0.into())),
      LiteralEntity::Symbol(_, _) => {
        // TODO: warn: TypeError: Cannot convert a Symbol value to a number
        None
      }
      LiteralEntity::NaN | LiteralEntity::Undefined => Some(None),
      LiteralEntity::Infinity(_) => None,
    }
  }

  fn get_prototype<'b>(&self, analyzer: &'b mut Analyzer<'a>) -> &'b Prototype<'a> {
    match self {
      LiteralEntity::String(_) => &analyzer.builtins.prototypes.string,
      LiteralEntity::Number(_, _) => &analyzer.builtins.prototypes.number,
      LiteralEntity::BigInt(_) => &analyzer.builtins.prototypes.bigint,
      LiteralEntity::Boolean(_) => &analyzer.builtins.prototypes.boolean,
      LiteralEntity::Symbol(_, _) => &analyzer.builtins.prototypes.symbol,
      LiteralEntity::Infinity(_) => &analyzer.builtins.prototypes.number,
      LiteralEntity::NaN => &analyzer.builtins.prototypes.number,
      LiteralEntity::Null | LiteralEntity::Undefined => unreachable!(),
    }
  }
}
