use ecma_analyzer::Analyzer;
use oxc::allocator::Allocator;
mod nodes;

pub struct TreeShaker<'a> {
  allocator: &'a Allocator,
}

impl<'a> Analyzer<'a> for TreeShaker<'a> {
  type Entity = &'a str;

  fn new_undefined(&self) -> Self::Entity {
    "undefined"
  }
}
