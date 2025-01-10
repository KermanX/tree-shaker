mod ast;
mod nodes;
mod scoping;

pub use ast::*;
pub use nodes::*;
pub use scoping::*;

pub trait EcmaAnalyzer<'a>:
  'a + ExpressionAnalyzer<'a> + StatementAnalyzer<'a> + ScopingAnalyzer<'a>
{
  type Entity;

  fn new_undefined_value(&self) -> Self::Entity
  where
    Self: EcmaAnalyzer<'a>;

  fn new_unknown_value(&self) -> Self::Entity
  where
    Self: EcmaAnalyzer<'a>;

  fn new_void_value(&self) -> Self::Entity
  where
    Self: EcmaAnalyzer<'a>;

  fn global_this(&self) -> Self::Entity
  where
    Self: EcmaAnalyzer<'a>;
}
