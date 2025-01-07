use super::{EntityHost, IntoEntity};

pub trait ArgumentsHost<'a>: EntityHost<'a> {
  type ArgumentsEntity: IntoEntity<'a, Self>;
}
