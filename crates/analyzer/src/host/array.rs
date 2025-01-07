use super::entity::{EntityHost, IntoEntity};
use oxc::ast::ast::*;

pub trait ArrayHost<'a>: EntityHost<'a> {
  type ArrayEntity: IntoEntity<'a, Self>;

  fn new_empty_array(&self, node: &'a ArrayExpression<'a>) -> Self::ArrayEntity;
  fn init_element(
    &self,
    node: &'a ArrayExpressionElement<'a>,
    array: &Self::ArrayEntity,
    value: Self::Entity,
  );
  fn init_spread(
    &self,
    node: &'a SpreadElement<'a>,
    array: &Self::ArrayEntity,
    value: Self::Entity,
  );
}
