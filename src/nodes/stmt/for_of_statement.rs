use crate::{
  analyzer::Analyzer, ast::AstKind2, consumable::box_consumable, scope::CfScopeKind,
  transformer::Transformer,
};
use oxc::{
  ast::ast::{ForOfStatement, Statement},
  span::GetSpan,
};

#[derive(Debug, Default, Clone)]
pub struct Data {
  need_loop: bool,
}

impl<'a> Analyzer<'a> {
  pub fn exec_for_of_statement(&mut self, node: &'a ForOfStatement<'a>) {
    let labels = self.take_labels();

    let dep = AstKind2::ForOfStatement(node);

    let right = self.exec_expression(&node.right);
    let right = if node.r#await {
      right.consume(self);
      self.refer_dep(dep);
      self.factory.immutable_unknown
    } else {
      right
    };

    self.declare_for_statement_left(&node.left);

    let iterated_0 = right.iterate_result_union(self, box_consumable(dep));
    if iterated_0.is_none() {
      return;
    }

    let data = self.load_data::<Data>(AstKind2::ForOfStatement(node));
    data.need_loop = true;

    self.push_cf_scope(CfScopeKind::BreakableWithoutLabel, labels.clone(), Some(false));
    self.exec_loop(move |analyzer| {
      if let Some(iterated) = right.iterate_result_union(analyzer, box_consumable(dep)) {
        analyzer.declare_for_statement_left(&node.left);
        analyzer.init_for_statement_left(&node.left, iterated);

        analyzer.push_cf_scope(CfScopeKind::Continuable, labels.clone(), None);
        analyzer.exec_statement(&node.body);
        analyzer.pop_cf_scope();
      }
    });
    self.pop_cf_scope();
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_for_of_statement(&self, node: &'a ForOfStatement<'a>) -> Option<Statement<'a>> {
    let data = self.get_data::<Data>(AstKind2::ForOfStatement(node));

    let ForOfStatement { span, r#await, left, right, body, .. } = node;

    let left_span = left.span();
    let body_span = body.span();

    let left = self.transform_for_statement_left(left);
    let body = self.transform_statement(body);

    if (!data.need_loop || (left.is_none() && body.is_none())) && !r#await {
      return if self.is_referred(AstKind2::ForOfStatement(node)) {
        let right_span = right.span();
        let right = self.transform_expression(right, true).unwrap();
        Some(
          self.ast_builder.statement_expression(
            *span,
            self.ast_builder.expression_array(
              *span,
              self
                .ast_builder
                .vec1(self.ast_builder.array_expression_element_spread_element(right_span, right)),
              None,
            ),
          ),
        )
      } else {
        self
          .transform_expression(right, false)
          .map(|expr| self.ast_builder.statement_expression(*span, expr))
      };
    }

    let right = self.transform_expression(right, true).unwrap();

    Some(self.ast_builder.statement_for_of(
      *span,
      *r#await,
      left.unwrap_or_else(|| self.build_unused_for_statement_left(left_span)),
      right,
      body.unwrap_or_else(|| self.ast_builder.statement_empty(body_span)),
    ))
  }
}
