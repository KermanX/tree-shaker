use crate::nodes::{expr::TraverseExpression, stmt::TraverseStatement};

#[allow(unused_variables)]
pub trait TraverseHost<'a>: TraverseExpression<'a> + TraverseStatement<'a> {}
