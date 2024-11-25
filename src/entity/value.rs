use super::{function::CalleeId, Entity, LiteralEntity, PrimitiveEntity};
use rustc_hash::FxHashSet;

pub enum EntityValueKind<'a> {
  Literal(LiteralEntity<'a>),
  Function(CalleeId<'a>),
  Primitive(PrimitiveEntity),
  Union(Vec<EntityValueKind<'a>>),
  AnyObject,
  Unknown,
}

pub struct MergedEntityValue<'a>(Option<Vec<EntityValueKind<'a>>>);

impl<'a> MergedEntityValue<'a> {
  pub fn contains(&self, value: &EntityValueKind<'a>) -> bool {
    if let Some(values) = &self.0 {
      values.iter().any(|v| v == value)
    } else {
      // Unknown
      true
    }
  }
}
