use crate::{
  builtins::Builtins,
  entity::{Entity, EntityFactory, EntityOpHost, EntityTrait, LabelEntity},
  mangling::Mangler,
  utils::{
    dep_id::{DepId, ReferredDeps},
    ExtraData,
  },
  TreeShakeConfig, TreeShaker,
};
use ecma_analyzer::{EcmaAnalyzer, Scoping};
use line_index::LineIndex;
use oxc::{
  allocator::Allocator,
  semantic::{Semantic, SymbolId},
  span::{GetSpan, Span},
};
use rustc_hash::FxHashSet;
use std::{mem, rc::Rc};

pub struct Analyzer<'a> {
  pub scoping: Scoping<'a, Self>,

  pub tree_shaker: TreeShaker<'a>,
  pub config: &'a TreeShakeConfig,
  pub allocator: &'a Allocator,
  pub factory: &'a EntityFactory<'a>,
  pub line_index: LineIndex,
  pub semantic: Semantic<'a>,
  pub span_stack: Vec<Span>,
  pub data: ExtraData<'a>,
  pub referred_deps: ReferredDeps,
  pub conditional_data: ConditionalDataMap<'a>,
  pub loop_data: LoopDataMap<'a>,
  pub mangler: Mangler<'a>,
  pub named_exports: Vec<SymbolId>,
  pub default_export: Option<Entity<'a>>,
  pub pending_deps: FxHashSet<ExhaustiveCallback<'a>>,
  pub builtins: Builtins<'a>,
  pub entity_op: EntityOpHost<'a>,

  pub debug: usize,
}

impl<'a> Analyzer<'a> {
  pub fn new(tree_shaker: TreeShaker<'a>, semantic: Semantic<'a>) -> Self {
    let config = tree_shaker.0.config;
    let allocator = tree_shaker.0.allocator;
    let factory = tree_shaker.0.factory;

    Analyzer {
      scoping: Scoping::new(),

      tree_shaker,
      config,
      allocator,
      factory,
      line_index: LineIndex::new(semantic.source_text()),
      semantic,
      span_stack: vec![],
      data: Default::default(),
      referred_deps: Default::default(),
      conditional_data: Default::default(),
      loop_data: Default::default(),
      mangler: Mangler::new(config.mangling, allocator),
      named_exports: Vec::new(),
      default_export: None,
      pending_deps: Default::default(),
      builtins: Builtins::new(config, factory),
      entity_op: EntityOpHost::new(allocator),
      debug: 0,
    }
  }
}

impl<'a> EcmaAnalyzer<'a> for Analyzer<'a> {
  type Entity = &'a (dyn EntityTrait<'a> + 'a);

  fn new_undefined_value(&self) -> Self::Entity {
    "undefined"
  }
}

impl<'a> Analyzer<'a> {
  pub fn set_data(&mut self, key: impl Into<DepId>, data: impl Default + 'a) {
    self.data.insert(key.into(), unsafe { mem::transmute(Box::new(data)) });
  }

  pub fn get_data_or_insert_with<D: 'a>(
    &mut self,
    key: impl Into<DepId>,
    default: impl FnOnce() -> D,
  ) -> &'a mut D {
    let boxed =
      self.data.entry(key.into()).or_insert_with(|| unsafe { mem::transmute(Box::new(default())) });
    unsafe { mem::transmute(boxed.as_mut()) }
  }

  pub fn load_data<D: Default + 'a>(&mut self, key: impl Into<DepId>) -> &'a mut D {
    self.get_data_or_insert_with(key, Default::default)
  }

  pub fn current_span(&self) -> Span {
    *self.span_stack.last().unwrap()
  }

  pub fn add_diagnostic(&mut self, message: impl Into<String>) {
    let span = self.current_span();
    let start = self.line_index.line_col(span.start.into());
    let end = self.line_index.line_col(span.end.into());
    let span_text =
      format!(" at {}:{}-{}:{}", start.line + 1, start.col + 1, end.line + 1, end.col + 1);
    self.tree_shaker.0.diagnostics.borrow_mut().insert(message.into() + &span_text);
  }

  pub fn push_span(&mut self, node: &'a impl GetSpan) {
    self.span_stack.push(node.span());
  }

  pub fn pop_span(&mut self) {
    self.span_stack.pop();
  }
}
