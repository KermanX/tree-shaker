use regex::Regex;

#[derive(Debug, Clone)]
pub struct TreeShakeConfig {
  pub unknown_global_side_effects: bool,
  pub preserve_function_length: bool,
  pub iterate_side_effects: bool,

  pub min_simple_number_value: i64,
  pub max_simple_number_value: i64,
  pub max_simple_string_length: usize,
  pub static_property_key_regex: Regex,
}

impl Default for TreeShakeConfig {
  fn default() -> Self {
    Self {
      unknown_global_side_effects: true,
      preserve_function_length: true,
      iterate_side_effects: true,

      min_simple_number_value: -999,
      max_simple_number_value: 999,
      max_simple_string_length: 12,
      static_property_key_regex: Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]+$").unwrap(),
    }
  }
}
