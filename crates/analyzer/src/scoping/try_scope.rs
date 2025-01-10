use crate::EcmaAnalyzer;

#[derive(Debug)]
pub struct TryScope<'a, A: EcmaAnalyzer<'a> + ?Sized> {
  pub may_throw: bool,
  pub thrown_values: Vec<A::Entity>,
  /// Here we use index in current stack instead of ScopeId
  pub cf_scope_depth: usize,
}

impl<'a, A: EcmaAnalyzer<'a> + ?Sized> TryScope<'a, A> {
  pub fn new(cf_scope_depth: usize) -> Self {
    TryScope { may_throw: false, thrown_values: Vec::new(), cf_scope_depth }
  }
}

pub trait TryScopeAnalyzer<'a> {
  fn get_thrown_val(&mut self, scope: TryScope<'a, Self>) -> Option<Self::Entity>
  where
    Self: EcmaAnalyzer<'a>;

  fn may_throw(&mut self)
  where
    Self: EcmaAnalyzer<'a>,
  {
    let try_scope = self.try_scope_mut();

    try_scope.may_throw = true;

    // FIXME: Some of the tests are failing because of this
    // let cf_scope_depth = try_scope.cf_scope_depth;
    // self.exit_to(cf_scope_depth, false);
  }

  fn explicit_throw(&mut self, value: Self::Entity)
  where
    Self: EcmaAnalyzer<'a>,
  {
    self.explicit_throw_impl(value);

    let try_scope = self.try_scope();
    self.exit_to(try_scope.cf_scope_depth);
  }

  fn thrown_builtin_error(&mut self, message: impl Into<String>)
  where
    Self: EcmaAnalyzer<'a>,
  {
    if self.scoping_mut().cf.iter_stack().all(|scope| scope.exited == Some(false)) {
      self.add_diagnostic(message);
    }

    self.explicit_throw_impl(self.factory.unknown());

    let try_scope = self.try_scope();
    self.exit_to(try_scope.cf_scope_depth);
  }

  fn forward_throw(&mut self, values: Vec<Self::Entity>)
  where
    Self: EcmaAnalyzer<'a>,
  {
    if values.is_empty() {
      self.may_throw();
    } else {
      let thrown_val = self.factory.computed_unknown(self.consumable(values));
      self.explicit_throw_impl(thrown_val);

      let try_scope = self.try_scope();
      self.exit_to_not_must(try_scope.cf_scope_depth);
    }
  }

  fn explicit_throw_impl(&mut self, value: Self::Entity)
  where
    Self: EcmaAnalyzer<'a>,
  {
    let try_scope = self.try_scope();
    let exec_dep = self.get_exec_dep(try_scope.cf_scope_depth);
    let forwarded = self.factory.computed(value, exec_dep);

    let try_scope = self.try_scope_mut();
    try_scope.may_throw = true;
    try_scope.thrown_values.push(forwarded);
  }
}
