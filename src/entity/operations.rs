use super::entity::Entity;
use oxc::allocator::Allocator;

pub(crate) struct EntityOpHost<'a> {
  allocator: &'a Allocator,
}

impl<'a> EntityOpHost<'a> {
  pub fn new(allocator: &'a Allocator) -> Self {
    Self { allocator }
  }

  pub fn add(&self, lhs: &Entity<'a>, rhs: &Entity<'a>) -> Entity<'a> {
    todo!()
  }
}
