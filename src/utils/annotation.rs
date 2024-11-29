use crate::{analyzer::Analyzer, dep::ReferredDeps};
use oxc::span::{GetSpan, Span};
use std::mem;

impl<'a> Analyzer<'a> {
  fn has_annotation(&self, span: Span, test: fn(&str) -> bool) -> bool {
    let Some(comment) = self.semantic.comments_range(..span.start).next_back() else {
      return false;
    };
    let raw = comment.span.source_text(self.semantic.source_text());

    // If there are non-whitespace characters between the `comment` and the `span`,
    // we treat the `comment` not belongs to the `span`.
    let range_text =
      Span::new(comment.span.end, span.start).source_text(self.semantic.source_text());

    // FIXME: WTF
    // let only_whitespace = match comment.kind {
    //   CommentKind::Line => range_text.trim().is_empty(),
    //   CommentKind::Block => {
    //     range_text
    //       .strip_prefix("*/") // for multi-line comment
    //       .is_some_and(|s| s.trim().is_empty())
    //   }
    // };
    let only_whitespace = range_text.trim().is_empty();
    if !only_whitespace {
      return false;
    }

    test(raw)
  }

  pub fn has_pure_notation(&self, node: &impl GetSpan) -> Option<Box<ReferredDeps>> {
    self
      .has_annotation(node.span(), |raw| raw.contains("@__PURE__") || raw.contains("#__PURE__"))
      .then(|| Box::new(ReferredDeps::default()))
  }

  pub fn exec_in_pure<T>(
    &mut self,
    pure_deps: Option<Box<ReferredDeps>>,
    runner: impl FnOnce(&mut Analyzer<'a>) -> T,
  ) -> (T, Option<Box<ReferredDeps>>) {
    if let Some(pure_deps) = pure_deps {
      let parent = mem::replace(&mut self.referred_deps, pure_deps);
      self.scope_context.pure += 1;
      let result = runner(self);
      self.scope_context.pure -= 1;
      (result, Some(mem::replace(&mut self.referred_deps, parent)))
    } else {
      let result = runner(self);
      (result, None)
    }
  }

  pub fn has_finite_recursion_notation(&self, node: impl GetSpan) -> bool {
    self.has_annotation(node.span(), |raw| {
      raw.contains("@__FINITE_RECURSION__") || raw.contains("#__FINITE_RECURSION__")
    })
  }
}
