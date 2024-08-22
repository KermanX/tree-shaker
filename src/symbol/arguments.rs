use crate::symbol::SymbolSource;

#[derive(Clone)]
pub struct ArgumentsEntity<'a> {
  args: Vec<(bool, SymbolSource<'a>)>,
}

impl<'a> ArgumentsEntity<'a> {
  pub(crate) fn new(args: Vec<(bool, SymbolSource<'a>)>) -> Self {
    ArgumentsEntity { args }
  }

  /// (args, rest)
  pub(crate) fn resolve(self, length: usize) -> (Vec<SymbolSource<'a>>, SymbolSource<'a>) {
    let mut resolved = vec![];
    for (expend, arg) in self.args {
      // TODO: handle expend
      assert!(!expend, "not implemented");
      resolved.push(arg);
    }

    for _ in resolved.len()..length {
      resolved.push(SymbolSource::Unknown);
    }

    (resolved, SymbolSource::Unknown)
  }
}
