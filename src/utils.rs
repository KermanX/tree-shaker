use regex::Regex;
use std::cell::LazyCell;


const NUMERIC_REGEX: LazyCell<Regex> = LazyCell::new(|| Regex::new(r"^[0-9]+$").unwrap());

pub fn is_numeric_string(s: &str) -> bool {
  NUMERIC_REGEX.is_match(s)
}
