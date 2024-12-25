#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeShakeJsxPreset {
  None,
  React,
}

impl TreeShakeJsxPreset {
  pub fn is_enabled(&self) -> bool {
    *self != Self::None
  }
}

#[derive(Debug, Clone)]
pub struct TreeShakeConfig {
  pub enabled: bool,
  pub jsx: TreeShakeJsxPreset,

  pub mangling: bool,
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
      enabled: true,
      jsx: TreeShakeJsxPreset::None,

      mangling: false,
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
    Self {
      mangling: true,
      preserve_function_name: false,
      preserve_function_length: false,
      ..Default::default()
    }
  }

  pub fn smallest() -> Self {
    Self {
      mangling: true,
      unknown_global_side_effects: false,
      preserve_function_name: false,
      preserve_function_length: false,
      iterate_side_effects: false,
      unknown_property_read_side_effects: false,

      ..Default::default()
    }
  }

  pub fn disabled() -> Self {
    Self { enabled: false, ..Default::default() }
  }

  pub fn with_react_jsx(mut self, yes: bool) -> Self {
    self.jsx = if yes { TreeShakeJsxPreset::React } else { TreeShakeJsxPreset::None };
    self
  }

  pub fn with_always_inline_literal(mut self, yes: bool) -> Self {
    if yes {
      self.min_simple_number_value = i64::MIN;
      self.max_simple_number_value = i64::MAX;
      self.max_simple_string_length = usize::MAX;
    }
    self
  }

  pub fn with_mangling(mut self, yes: bool) -> Self {
    self.mangling = yes;
    self
  }
}
