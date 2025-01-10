use crate::{entity::Entity, Analyzer};
use ecma_analyzer::{CallScope, CallScopeAnalyzer};

impl<'a> CallScopeAnalyzer<'a> for Analyzer<'a> {
  fn get_return_value(&mut self, scope: CallScope<'a, Self>) -> Self::Entity
  where
    Self: ecma_analyzer::EcmaAnalyzer<'a>,
  {
    // Forwards the thrown value to the parent try scope
    let try_scope = scope.try_scopes.into_iter().next().unwrap();
    let mut promise_error = None;
    if try_scope.may_throw {
      if scope.is_generator {
        let unknown = self.new_unknown_value();
        let parent_try_scope = self.try_scope_mut();
        parent_try_scope.may_throw = true;
        if !try_scope.thrown_values.is_empty() {
          parent_try_scope.thrown_values.push(unknown);
        }
        self.consume(try_scope.thrown_values);
      } else if scope.is_async {
        promise_error = Some(try_scope.thrown_values);
      } else {
        self.forward_throw(try_scope.thrown_values);
      }
    }

    let value = if scope.returned_values.is_empty() {
      self.new_undefined_value()
    } else {
      self.factory.union(scope.returned_values)
    };

    if scope.is_async {
      self.factory.computed_unknown(self.consumable((value, promise_error)))
    } else {
      value
    };
  }
}
