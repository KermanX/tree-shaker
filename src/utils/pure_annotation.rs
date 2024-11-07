use crate::analyzer::Analyzer;
use oxc::{ast::CommentKind, span::Span};

impl<'a> Analyzer<'a> {
  pub fn has_pure_notation(&self, span: Span) -> usize {
    let Some(comment) = self.semantic.comments_range(..span.start).next_back() else {
      return 0;
    };
    let raw = comment.span.source_text(self.semantic.source_text());

    // If there are non-whitespace characters between the `comment` and the `span`,
    // we treat the `comment` not belongs to the `span`.
    let range_text =
      Span::new(comment.span.end, span.start).source_text(self.semantic.source_text());
    let only_whitespace = match comment.kind {
      CommentKind::Line => range_text.trim().is_empty(),
      CommentKind::Block => {
        range_text
          .strip_prefix("*/") // for multi-line comment
          .is_some_and(|s| s.trim().is_empty())
      }
    };
    if !only_whitespace {
      return 0;
    }

    if raw.contains("@__PURE__") || raw.contains("#__PURE__") {
      1
    } else {
      0
    }
  }
}
