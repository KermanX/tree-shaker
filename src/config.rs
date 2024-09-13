#[derive(Debug, Clone)]
pub struct TreeShakeConfig {
  pub unknown_global_side_effects: bool,

  pub min_simple_number_value: i64,
  pub max_simple_number_value: i64,
  pub max_simple_string_length: usize,
}

impl Default for TreeShakeConfig {
  fn default() -> Self {
    Self {
      unknown_global_side_effects: true,

      min_simple_number_value: -999,
      max_simple_number_value: 999,
      max_simple_string_length: 12,
    }
  }
}
