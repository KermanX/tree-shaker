use crate::TreeShaker;
use ecma_analyzer::ExpressionAnalyzer;

mod array_expression;
mod literals;

impl<'a> ExpressionAnalyzer<'a> for TreeShaker<'a> {}
