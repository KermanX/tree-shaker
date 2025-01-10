mod call_scope;

use crate::analyzer::Analyzer;
use ecma_analyzer::{EcmaAnalyzer, Scoping, ScopingAnalyzer};

impl<'a> ScopingAnalyzer<'a> for Analyzer<'a> {
  fn scoping() -> &'a Scoping<'a, Self>
  where
    Self: EcmaAnalyzer<'a>,
  {
    &self.scoping
  }

  fn scoping_mut(&mut self) -> &mut Scoping<'a, Self>
  where
    Self: EcmaAnalyzer<'a>,
  {
    &mut self.scoping
  }

  fn init_scoping(&mut self) {
      
  }
}
