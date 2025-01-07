use super::entity::EntityHost;
use oxc_syntax::operator::BinaryOperator;

pub trait EvaluationHost<'a>: EntityHost<'a> {
  fn test_truthy(&self, value: Self::Entity) -> Option<bool>;
  fn test_nullish(&self, value: Self::Entity) -> Option<bool>;

  fn to_boolean(&self, value: Self::Entity) -> Self::Entity;
  fn binary_op(
    &self,
    operator: BinaryOperator,
    lhs: Self::Entity,
    rhs: Self::Entity,
  ) -> Self::Entity;
  fn awaited(&self, value: Self::Entity) -> Self::Entity;
}
