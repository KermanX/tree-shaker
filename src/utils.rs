use regex::Regex;
use std::sync::LazyLock;

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
