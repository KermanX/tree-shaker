use super::source::SymbolSource;
use crate::ast::Arguments;

pub(crate) trait ArgumentsSource<'a> {
  fn resolve(&self, length: usize) -> (Vec<SymbolSource<'a>>, SymbolSource<'a>);
}

pub(crate) struct ArgumentsSourceFromNode<'a> {
  pub(crate) node: &'a Arguments<'a>,
}

impl<'a> ArgumentsSource<'a> for ArgumentsSourceFromNode<'a> {
  fn resolve(&self, length: usize) -> (Vec<SymbolSource<'a>>, SymbolSource<'a>) {
    todo!()
  }
}

pub(crate) struct ArgumentsSourceUnknown {}

impl<'a> ArgumentsSource<'a> for ArgumentsSourceUnknown {
  fn resolve(&self, length: usize) -> (Vec<SymbolSource<'a>>, SymbolSource<'a>) {
    (vec![SymbolSource::Unknown; length], SymbolSource::Unknown)
  }
}
