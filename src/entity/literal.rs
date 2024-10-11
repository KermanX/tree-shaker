use super::{
  consumed_object, entity::EnumeratedProperties, Entity, EntityFactory, EntityTrait, TypeofResult,
};
use crate::{analyzer::Analyzer, builtins::Prototype, consumable::Consumable, utils::F64WithEq};
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
  fn consume(&self, _analyzer: &mut Analyzer<'a>) {}

  fn get_property(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
  ) -> Entity<'a> {
    if matches!(self, LiteralEntity::Null | LiteralEntity::Undefined) {
      analyzer.thrown_builtin_error("Cannot get property of null or undefined");
      consumed_object::get_property(rc, analyzer, dep, key)
    } else {
      let prototype = self.get_prototype(analyzer);
      prototype.get_property(analyzer, rc, key, dep)
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
    if matches!(self, LiteralEntity::Null | LiteralEntity::Undefined) {
      analyzer.thrown_builtin_error("Cannot set property of null or undefined");
      consumed_object::set_property(analyzer, dep, key, value)
    } else {
      // No effect
    }
  }

  fn enumerate_properties(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> EnumeratedProperties<'a> {
    if let LiteralEntity::String(value) = self {
      if value.len() <= analyzer.config.max_simple_string_length {
        (
          value
            .char_indices()
            .map(|(i, c)| {
              (
                true,
                analyzer.factory.new_string(analyzer.allocator.alloc(i.to_string())),
                analyzer.factory.new_string(analyzer.allocator.alloc(c.to_string())),
              )
            })
            .collect(),
          dep,
        )
      } else {
        analyzer
          .factory
          .new_computed_unknown_string(rc.to_consumable())
          .enumerate_properties(analyzer, dep)
      }
    } else {
      // No effect
      (vec![], dep)
    }
  }

  fn delete_property(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, _key: Entity<'a>) {
    if matches!(self, LiteralEntity::Null | LiteralEntity::Undefined) {
      analyzer.thrown_builtin_error("Cannot delete property of null or undefined");
      analyzer.consume(dep);
    } else {
      // No effect
    }
  }

  fn call(
    &self,
    _rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    this: Entity<'a>,
    args: Entity<'a>,
  ) -> Entity<'a> {
    analyzer.thrown_builtin_error(format!("Cannot call a non-function object {:?}", self));
    consumed_object::call(analyzer, dep, this, args)
  }

  fn r#await(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> Entity<'a> {
    analyzer.factory.new_computed(rc, dep)
  }

  fn iterate(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
  ) -> (Vec<Entity<'a>>, Option<Entity<'a>>) {
    match self {
      LiteralEntity::String(value) => (
        vec![],
        if value.is_empty() {
          None
        } else {
          Some(analyzer.factory.new_computed_unknown_string(rc.to_consumable()))
        },
      ),
      _ => {
        self.consume(analyzer);
        analyzer.thrown_builtin_error("Cannot iterate over a non-iterable object");
        consumed_object::iterate(analyzer, dep)
      }
    }
  }

  fn get_typeof(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    analyzer.factory.new_string(self.test_typeof().to_string().unwrap())
  }

  fn get_to_string(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    analyzer.factory.new_string(self.to_string())
  }

  fn get_to_numeric(&self, rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    match self {
      LiteralEntity::Number(_, _)
      | LiteralEntity::BigInt(_)
      | LiteralEntity::NaN
      | LiteralEntity::Infinity(_) => rc,
      LiteralEntity::Boolean(value) => {
        if *value {
          analyzer.factory.new_number(1.0, "1")
        } else {
          analyzer.factory.new_number(0.0, "0")
        }
      }
      LiteralEntity::String(str) => {
        let str = str.trim();
        if str.is_empty() {
          analyzer.factory.new_number(0.0, "0")
        } else {
          if let Ok(value) = str.parse::<f64>() {
            analyzer.factory.new_number(value, str)
          } else {
            analyzer.factory.nan
          }
        }
      }
      LiteralEntity::Null => analyzer.factory.new_number(0.0, "0"),
      LiteralEntity::Symbol(_, _) => {
        // TODO: warn: TypeError: Cannot convert a Symbol value to a number
        analyzer.factory.unknown
      }
      LiteralEntity::Undefined => analyzer.factory.nan,
    }
  }

  fn get_to_boolean(&self, rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    match self.test_truthy() {
      Some(value) => analyzer.factory.new_boolean(value),
      None => analyzer.factory.new_computed_unknown_boolean(rc.to_consumable()),
    }
  }

  fn get_to_property_key(&self, rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Entity<'a> {
    match self {
      LiteralEntity::Symbol(_, _) => rc,
      _ => self.get_to_string(rc, analyzer),
    }
  }

  fn get_to_literals(
    &self,
    _rc: Entity<'a>,
    analyzer: &Analyzer<'a>,
  ) -> Option<FxHashSet<LiteralEntity<'a>>> {
    let mut result = FxHashSet::default();
    result.insert(*self);
    Some(result)
  }

  fn get_literal(&self, _rc: Entity<'a>, analyzer: &Analyzer<'a>) -> Option<LiteralEntity<'a>> {
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
      LiteralEntity::Number(value, _) => *value != 0.0.into() && *value != (-0.0).into(),
      LiteralEntity::BigInt(value) => !value.chars().all(|c| c == '0'),
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
  pub fn build_expr(&self, ast_builder: &AstBuilder<'a>, span: Span) -> Expression<'a> {
    match self {
      LiteralEntity::String(value) => ast_builder.expression_string_literal(span, *value),
      LiteralEntity::Number(value, raw) => {
        let negated = raw.chars().nth(0).unwrap() == '-';
        let absolute = ast_builder.expression_numeric_literal(
          span,
          value.0.abs(),
          if negated { &raw[1..] } else { raw },
          NumberBase::Decimal,
        );
        if negated {
          ast_builder.expression_unary(span, UnaryOperator::UnaryNegation, absolute)
        } else {
          absolute
        }
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
      LiteralEntity::Number(_, str_rep) => *str_rep,
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
        let value = value.trim();
        Some(if value.is_empty() {
          Some(0.0.into())
        } else {
          if let Ok(value) = value.parse::<f64>() {
            Some(value.into())
          } else {
            None
          }
        })
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

  fn get_prototype<'b>(&self, analyzer: &mut Analyzer<'a>) -> &'a Prototype<'a> {
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

impl<'a> EntityFactory<'a> {
  pub fn new_string(&self, value: &'a str) -> Entity<'a> {
    self.new_entity(LiteralEntity::String(value))
  }

  pub fn new_number(&self, value: impl Into<F64WithEq>, str_rep: &'a str) -> Entity<'a> {
    self.new_entity(LiteralEntity::Number(value.into(), str_rep))
  }

  pub fn new_big_int(&self, value: &'a str) -> Entity<'a> {
    self.new_entity(LiteralEntity::BigInt(value))
  }

  pub fn new_boolean(&self, value: bool) -> Entity<'a> {
    self.new_entity(LiteralEntity::Boolean(value))
  }

  pub fn new_infinity(&self, positive: bool) -> Entity<'a> {
    self.new_entity(LiteralEntity::Infinity(positive))
  }

  pub fn new_symbol(&self, id: SymbolId, str_rep: &'a str) -> Entity<'a> {
    self.new_entity(LiteralEntity::Symbol(id, str_rep))
  }
}
