use crate::{symbol::SymbolSource, utils::DataPlaceholder};
use oxc::{
  allocator::Allocator,
  ast::ast::{Function, Program},
  semantic::{Semantic, SymbolId},
  span::{GetSpan, Span},
};
use rustc_hash::FxHashMap;
use std::mem;

pub(crate) struct Analyzer<'a> {
  allocator: &'a Allocator,
  pub sematic: Semantic<'a>,
  pub functions: FxHashMap<Span, &'a Function<'a>>,
  pub symbol_source: FxHashMap<SymbolId, SymbolSource<'a>>,
  pub data: FxHashMap<Span, Box<DataPlaceholder<'a>>>,
  pub indeterminate: bool,
}

impl<'a> Analyzer<'a> {
  pub fn new(allocator: &'a Allocator, sematic: Semantic<'a>) -> Self {
    Analyzer {
      allocator,
      sematic,
      functions: FxHashMap::default(),
      symbol_source: FxHashMap::default(),
      data: FxHashMap::default(),
      indeterminate: false,
    }
  }

  pub fn exec_program(&mut self, ast: &'a Program<'a>) {
    for statement in &ast.body {
      self.exec_statement(statement);
    }
  }
}

impl<'a> Analyzer<'a> {
  pub(crate) fn set_data_by_span(&mut self, span: Span, data: impl Default + 'a) {
    self.data.insert(span, unsafe { mem::transmute(Box::new(data)) });
  }

  pub(crate) fn set_data(&mut self, node: &dyn GetSpan, data: impl Default + 'a) {
    self.set_data_by_span(node.span(), data)
  }

  pub(crate) fn load_data_by_span<D: Default + 'a>(&mut self, span: Span) -> &'a mut D {
    let existing = self.data.get_mut(&span);
    match existing {
      Some(boxed) => unsafe { mem::transmute(boxed.as_mut()) },
      None => {
        let data = D::default();
        self.set_data_by_span(span, data);
        self.load_data_by_span(span)
      }
    }
  }

  pub(crate) fn load_data<D: Default + 'a>(&mut self, node: &dyn GetSpan) -> &'a mut D {
    self.load_data_by_span(node.span())
  }

  pub(crate) fn get_data_by_span<D: Default + 'a>(&self, span: Span) -> &'a D {
    let existing = self.data.get(&span);
    match existing {
      Some(boxed) => unsafe { mem::transmute(boxed.as_ref()) },
      None => self.allocator.alloc(D::default()),
    }
  }

  pub(crate) fn get_data<D: Default + 'a>(&self, node: &dyn GetSpan) -> &'a D {
    self.get_data_by_span(node.span())
  }
}

pub(crate) struct IndeterminateBackup(bool);

impl<'a> Analyzer<'a> {
  pub(crate) fn start_indeterminate(&mut self) -> IndeterminateBackup {
    let prev = self.indeterminate;
    self.indeterminate = true;
    IndeterminateBackup(prev)
  }

  pub(crate) fn end_indeterminate(&mut self, prev: IndeterminateBackup) {
    self.indeterminate = prev.0;
  }
}
