pub trait IntoEntity<'a, H: EntityHost<'a> + ?Sized> {
  fn into_entity(self, host: &'a H) -> H::Entity;
}

pub trait EntityHost<'a> {
  type Entity: IntoEntity<'a, Self>;

  fn to_entity<T: IntoEntity<'a, Self>>(&self, value: T) -> Self::Entity {
    value.into_entity(self)
  }
}
