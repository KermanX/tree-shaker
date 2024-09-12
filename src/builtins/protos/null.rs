use super::Prototype;

pub fn create_null_prototype<'a>() -> Prototype<'a> {
  Prototype::new()
}
