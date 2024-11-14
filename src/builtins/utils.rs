#[macro_export]
macro_rules! init_namespace {
  ($ns:expr, { $($k:expr => $v:expr,)* }) => {
    {
      use $crate::entity::{ObjectProperty, ObjectPropertyValue};
      let mut string_keyed = $ns.string_keyed.borrow_mut();
      $(string_keyed.insert(
        $k,
        ObjectProperty {
          definite: true,
          values: vec![ObjectPropertyValue::Field($v, Some(true))],
        },
      );)*
    }
  };
}
