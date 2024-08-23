use oxc::ast::ast::{
  DoWhileStatement, ForInStatement, ForOfStatement, ForStatement, WhileStatement,
};

#[derive(Debug)]
pub(crate) enum LoopScopeNode<'a> {
  While(&'a WhileStatement<'a>),
  DoWhile(&'a DoWhileStatement<'a>),
  For(&'a ForStatement<'a>),
  ForIn(&'a ForInStatement<'a>),
  ForOf(&'a ForOfStatement<'a>),
}

#[derive(Debug)]
pub(crate) struct LoopScope<'a> {
  pub(crate) node: LoopScopeNode<'a>,
  pub(crate) broken: Option<bool>,
  pub(crate) continued: Option<bool>,
}
