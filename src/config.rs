#[derive(Debug, Clone)]
pub struct TreeShakeConfig {
  pub unknown_global_side_effects: bool,
  pub preserve_function_name: bool,
  pub preserve_function_length: bool,
  pub iterate_side_effects: bool,
  pub unknown_property_read_side_effects: bool,

  pub min_simple_number_value: i64,
  pub max_simple_number_value: i64,
  pub max_simple_string_length: usize,
}

impl Default for TreeShakeConfig {
  fn default() -> Self {
    Self::safest()
  }
}

impl TreeShakeConfig {
  pub fn safest() -> Self {
    Self {
      unknown_global_side_effects: true,
      preserve_function_name: true,
      preserve_function_length: true,
      iterate_side_effects: true,
      unknown_property_read_side_effects: true,

      min_simple_number_value: -999,
      max_simple_number_value: 999,
      max_simple_string_length: 12,
    }
  }

  pub fn recommended() -> Self {
    Self { preserve_function_name: false, preserve_function_length: false, ..Default::default() }
  }

  pub fn smallest() -> Self {
    Self {
      unknown_global_side_effects: false,
      preserve_function_name: false,
      preserve_function_length: false,
      iterate_side_effects: false,
      unknown_property_read_side_effects: false,

      ..Default::default()
    }
  }
}
