use crate::ast_type::AstType2;
use oxc::span::Span;
use regex::Regex;
use rustc_hash::FxHashMap;
use std::sync::LazyLock;

static NUMERIC_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[0-9]+$").unwrap());

pub(crate) fn is_numeric_string(s: &str) -> bool {
  NUMERIC_REGEX.is_match(s)
}

pub(crate) struct DataPlaceholder<'a> {
  _phantom: std::marker::PhantomData<&'a ()>,
}

pub(crate) type ExtraData<'a> = FxHashMap<AstType2, FxHashMap<Span, Box<DataPlaceholder<'a>>>>;
