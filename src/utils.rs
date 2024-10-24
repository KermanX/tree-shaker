use crate::analyzer::Analyzer;
use oxc::{ast::CommentKind, span::Span};
use std::hash::{Hash, Hasher};

#[derive(Debug, Copy, Clone)]
pub struct F64WithEq(pub f64);

impl PartialEq<Self> for F64WithEq {
  fn eq(&self, rhs: &Self) -> bool {
    self.0.to_le_bytes() == rhs.0.to_le_bytes()
  }
}

impl Hash for F64WithEq {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.0.to_le_bytes().hash(state)
  }
}

impl From<f64> for F64WithEq {
  fn from(val: f64) -> Self {
    Self(val)
  }
}

impl Into<f64> for F64WithEq {
  fn into(self) -> f64 {
    self.0
  }
}

impl Eq for F64WithEq {}

impl<'a> Analyzer<'a> {
  pub fn escape_private_identifier_name(&self, name: &str) -> &'a str {
    self.allocator.alloc(format!("__#private__{}", name))
  }

  pub fn has_pure_notation(&self, _span: Span) -> usize {
    return 0;

    // TODO: Pure annotation
    // let Some(comment) = self.semantic.comments_range(..span.start).next_back() else {
    //   return 0;
    // };
    // let raw = comment.span.source_text(self.semantic.source_text());

    // // If there are non-whitespace characters between the `comment` and the `span`,
    // // we treat the `comment` not belongs to the `span`.
    // let range_text =
    //   Span::new(comment.span.end, span.start).source_text(self.semantic.source_text());
    // let only_whitespace = match comment.kind {
    //   CommentKind::Line => range_text.trim().is_empty(),
    //   CommentKind::Block => {
    //     range_text
    //       .strip_prefix("*/") // for multi-line comment
    //       .is_some_and(|s| s.trim().is_empty())
    //   }
    // };
    // if !only_whitespace {
    //   return 0;
    // }

    // if raw.contains("@__PURE__") || raw.contains("#__PURE__") {
    //   1
    // } else {
    //   0
    // }
  }
}
