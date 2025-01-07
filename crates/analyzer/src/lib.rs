mod nodes;

pub use nodes::*;

pub trait Analyzer<'a>: ExpressionAnalyzer<'a> {
  type Entity;

  fn new_undefined(&self) -> Self::Entity
  where
    Self: Analyzer<'a>;
}
