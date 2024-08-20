use regex::Regex;
use std::cell::LazyCell;

#[macro_export]
macro_rules! build_expression_sequence {
  ($builder:expr, $span:expr, $($x:expr),+ $(,)?) => {
    {
      let mut exprs = $builder.vec();
      $($x.map(|e| exprs.push(e));)*
      if exprs.is_empty() {
        None
      }
      else if exprs.len() == 1 {
        exprs.pop()
      }
      else {
        Some($builder.expression_sequence($span, exprs))
      }
    }
  };
}

const NUMERIC_REGEX: LazyCell<Regex> = LazyCell::new(|| Regex::new(r"^[0-9]+$").unwrap());

pub fn is_numeric_string(s: &str) -> bool {
  NUMERIC_REGEX.is_match(s)
}
