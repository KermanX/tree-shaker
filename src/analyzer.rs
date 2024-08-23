use crate::{ast::AstType2, entity::source::SymbolSource, scope::ScopeContext, utils::ExtraData};
use oxc::{
  allocator::Allocator,
  ast::ast::{Function, Program},
  semantic::{Semantic, SymbolId},
  span::{GetSpan, Span},
};
use rustc_hash::FxHashMap;
use std::mem;

pub(crate) struct Analyzer<'a> {
  pub(crate) allocator: &'a Allocator,
  pub(crate) sematic: Semantic<'a>,
  pub(crate) functions: FxHashMap<Span, &'a Function<'a>>,
  pub(crate) symbol_source: FxHashMap<SymbolId, SymbolSource<'a>>,
  pub(crate) data: ExtraData<'a>,
  pub(crate) indeterminate: bool,
  pub(crate) exporting: bool,
  pub(crate) exports: Vec<SymbolId>,
  pub(crate) scope_context: ScopeContext<'a>,
}

impl<'a> Analyzer<'a> {
  pub(crate) fn new(allocator: &'a Allocator, sematic: Semantic<'a>) -> Self {
    Analyzer {
      allocator,
      sematic,
      functions: FxHashMap::default(),
      symbol_source: FxHashMap::default(),
      data: FxHashMap::default(),
      indeterminate: false,
      exporting: false,
      exports: Vec::new(),
      scope_context: ScopeContext::new(),
    }
  }

  pub(crate) fn exec_program(&mut self, ast: &'a Program<'a>) {
    for statement in &ast.body {
      self.exec_statement(statement);
    }

    for symbol in self.exports.clone() {
      // TODO: Should consume the symbol
      self.read_symbol(&symbol);
    }
  }
}

impl<'a> Analyzer<'a> {
  pub(crate) fn set_data_by_span(
    &mut self,
    ast_type: AstType2,
    span: Span,
    data: impl Default + 'a,
  ) {
    let map = self.data.entry(ast_type).or_insert_with(|| FxHashMap::default());
    map.insert(span, unsafe { mem::transmute(Box::new(data)) });
  }

  pub(crate) fn set_data(
    &mut self,
    ast_type: AstType2,
    node: &dyn GetSpan,
    data: impl Default + 'a,
  ) {
    self.set_data_by_span(ast_type, node.span(), data)
  }

  pub(crate) fn load_data_by_span<D: Default + 'a>(
    &mut self,
    ast_type: AstType2,
    span: Span,
  ) -> &'a mut D {
    let map = self.data.entry(ast_type).or_insert_with(|| FxHashMap::default());
    let boxed =
      map.entry(span).or_insert_with(|| unsafe { mem::transmute(Box::new(D::default())) });
    unsafe { mem::transmute(boxed.as_mut()) }
  }

  pub(crate) fn load_data<D: Default + 'a>(
    &mut self,
    ast_type: AstType2,
    node: &dyn GetSpan,
  ) -> &'a mut D {
    self.load_data_by_span(ast_type, node.span())
  }

  pub(crate) fn get_data_by_span<D: Default + 'a>(&self, ast_type: AstType2, span: Span) -> &'a D {
    let existing = self.data.get(&ast_type).and_then(|map| map.get(&span));
    match existing {
      Some(boxed) => unsafe { mem::transmute(boxed.as_ref()) },
      None => self.allocator.alloc(D::default()),
    }
  }

  pub(crate) fn get_data<D: Default + 'a>(&self, ast_type: AstType2, node: &dyn GetSpan) -> &'a D {
    self.get_data_by_span(ast_type, node.span())
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
