mod nodes;
mod scoping;

pub use nodes::*;

pub trait EcmaAnalyzer<'a>: ExpressionAnalyzer<'a> {
  type Entity;

  fn new_undefined(&self) -> Self::Entity
  where
    Self: EcmaAnalyzer<'a>;
}
