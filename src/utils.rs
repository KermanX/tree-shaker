use oxc::{
  ast::{
    ast::{
      CallExpression, Expression, ExpressionStatement, Program, Statement,
      TSTypeParameterInstantiation,
    },
    AstBuilder,
  },
  span::SPAN,
};
use regex::Regex;
use std::{
  hash::{Hash, Hasher},
  sync::LazyLock,
};

static NUMERIC_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[0-9]+$").unwrap());

pub fn is_numeric_string(s: &str) -> bool {
  NUMERIC_REGEX.is_match(s)
}

#[derive(Debug, Copy, Clone)]
pub struct F64WithEq(pub f64);

impl PartialEq<Self> for F64WithEq {
  fn eq(&self, rhs: &Self) -> bool {
    self.0.to_le_bytes() == rhs.0.to_le_bytes()
  }
}

impl Hash for F64WithEq {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.0.to_le_bytes().hash(state)
  }
}

impl From<f64> for F64WithEq {
  fn from(val: f64) -> Self {
    Self(val)
  }
}

impl Into<f64> for F64WithEq {
  fn into(self) -> f64 {
    self.0
  }
}

impl Eq for F64WithEq {}

const EVAL_MOD_RET_FN: &str = "__EVAL_RET__";

pub fn transform_eval_mode_encode<'a>(ast_builder: &AstBuilder<'a>, program: &mut Program<'a>) {
  let last = program.body.pop().unwrap();
  if let Statement::ExpressionStatement(stmt) = last {
    let ExpressionStatement { span, expression, .. } = stmt.unbox();
    let last = ast_builder.statement_expression(
      span,
      ast_builder.expression_call(
        span,
        ast_builder.expression_identifier_reference(SPAN, EVAL_MOD_RET_FN),
        None::<TSTypeParameterInstantiation>,
        ast_builder.vec1(ast_builder.argument_expression(expression)),
        false,
      ),
    );
    program.body.push(last);
  } else {
    todo!();
  }
}

pub fn transform_eval_mode_decode<'a>(ast_builder: &AstBuilder<'a>, program: &mut Program<'a>) {
  let last = program.body.pop().unwrap();
  if let Statement::ExpressionStatement(stmt) = last {
    let ExpressionStatement { span, expression, .. } = stmt.unbox();
    if let Expression::CallExpression(expression) = expression {
      let CallExpression { callee, mut arguments, .. } = expression.unbox();
      assert!({
        if let Expression::Identifier(id) = &callee {
          id.name == EVAL_MOD_RET_FN
        } else {
          false
        }
      });
      assert!(arguments.len() == 1);
      let argument = arguments.pop().unwrap();
      program.body.push(ast_builder.statement_expression(span, argument.try_into().unwrap()));
    }
  } else {
    todo!();
  }
}
