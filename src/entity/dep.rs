use core::hash::Hash;
use oxc::ast::AstKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityDep(u128);

impl<'a> From<AstKind<'a>> for EntityDep {
  fn from(node: AstKind<'a>) -> Self {
    EntityDep(unsafe { std::mem::transmute(node) })
  }
}

pub const ENVIRONMENT_DEP: EntityDep = EntityDep(0);
