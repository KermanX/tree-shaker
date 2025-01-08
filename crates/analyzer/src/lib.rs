mod ast;
mod nodes;
mod scoping;

pub use ast::*;
pub use nodes::*;
pub use scoping::*;

pub trait EcmaAnalyzer<'a>: ExpressionAnalyzer<'a> + StatementAnalyzer<'a> + ScopingAnalyzer<'a> {
  type Entity;

  fn new_undefined(&self) -> Self::Entity
  where
    Self: EcmaAnalyzer<'a>;
}
