use crate::{ast::AstType2, entity::dep::EntityDepNode};
use oxc::span::Span;
use rustc_hash::{FxHashMap, FxHashSet};

pub struct DataPlaceholder<'a> {
  _phantom: std::marker::PhantomData<&'a ()>,
}

pub type ExtraData<'a> = FxHashMap<AstType2, FxHashMap<Span, Box<DataPlaceholder<'a>>>>;

pub type ReferredNodes<'a> = FxHashSet<EntityDepNode<'a>>;
