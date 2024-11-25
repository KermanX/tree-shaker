use crate::{
  ast::AstKind2,
  builtins::Builtins,
  dep::{DepId, ReferredDeps},
  entity::{Entity, EntityFactory, EntityOpHost, LabelEntity},
  scope::{
    conditional::ConditionalDataMap, exhaustive::TrackerRunner, r#loop::LoopDataMap, ScopeContext,
  },
  tree_shaker::TreeShaker,
  utils::{DebuggerEvent, ExtraData, Logger, StatementVecData},
  TreeShakeConfig,
};
use oxc::{
  allocator::Allocator,
  ast::ast::Program,
  semantic::{Semantic, SymbolId},
  span::{GetSpan, Span},
};
use rustc_hash::FxHashSet;
use std::{mem, rc::Rc};

pub struct Analyzer<'a> {
  pub tree_shaker: TreeShaker<'a>,
  pub config: &'a TreeShakeConfig,
  pub allocator: &'a Allocator,
  pub factory: &'a EntityFactory<'a>,
  pub semantic: Semantic<'a>,
  pub span_stack: Vec<Span>,
  pub data: ExtraData<'a>,
  pub referred_deps: ReferredDeps,
  pub conditional_data: ConditionalDataMap<'a>,
  pub loop_data: LoopDataMap<'a>,
  pub named_exports: Vec<SymbolId>,
  pub default_export: Option<Entity<'a>>,
  pub scope_context: ScopeContext<'a>,
  pub pending_labels: Vec<LabelEntity<'a>>,
  pub pending_deps: FxHashSet<TrackerRunner<'a>>,
  pub builtins: Builtins<'a>,
  pub entity_op: EntityOpHost<'a>,
  pub logger: Option<&'a Logger>,

  pub debug: usize,
}

impl<'a> Analyzer<'a> {
  pub fn new(tree_shaker: TreeShaker<'a>, semantic: Semantic<'a>) -> Self {
    let config = tree_shaker.0.config;
    let allocator = tree_shaker.0.allocator;
    let factory = tree_shaker.0.factory;
    let logger = tree_shaker.0.logger;

    Analyzer {
      tree_shaker,
      config,
      allocator,
      factory,
      semantic,
      span_stack: vec![],
      data: Default::default(),
      referred_deps: Default::default(),
      conditional_data: Default::default(),
      loop_data: Default::default(),
      named_exports: Vec::new(),
      default_export: None,
      scope_context: ScopeContext::new(&factory),
      pending_labels: Vec::new(),
      pending_deps: Default::default(),
      builtins: Builtins::new(config, factory),
      entity_op: EntityOpHost::new(allocator),
      logger,
      debug: 0,
    }
  }

  pub fn exec_program(&mut self, node: &'a Program<'a>) {
    // Top level is always preserved
    let top_level_call_id = self.call_scope().call_id;
    self.refer_dep(top_level_call_id);

    let data = self.load_data::<StatementVecData>(AstKind2::Program(node));
    self.exec_statement_vec(data, &node.body);

    self.consume_exports();

    let mut round = 0usize;
    loop {
      round += 1;
      if round > 1000 {
        panic!("Possible infinite loop in post analysis");
      }

      let mut dirty = false;
      dirty |= self.consume_top_level_uncaught();
      dirty |= self.call_exhaustive_deps();
      dirty |= self.post_analyze_handle_conditional();
      dirty |= self.post_analyze_handle_loops();
      if !dirty {
        break;
      }
    }

    self.scope_context.assert_final_state();

    println!("debug: {:?}", self.debug);
  }

  pub fn consume_exports(&mut self) {
    self.default_export.take().map(|entity| entity.consume(self));
    for symbol in self.named_exports.clone() {
      let entity = self.read_symbol(symbol).unwrap();
      entity.consume(self);
    }
  }

  pub fn consume_top_level_uncaught(&mut self) -> bool {
    let thrown_values = &mut self.call_scope_mut().try_scopes.last_mut().unwrap().thrown_values;
    if thrown_values.is_empty() {
      false
    } else {
      let values = mem::take(thrown_values);
      self.consume(values);
      true
    }
  }
}

impl<'a> Analyzer<'a> {
  pub fn set_data(&mut self, key: impl Into<DepId>, data: impl Default + 'a) {
    self.data.insert(key.into(), unsafe { mem::transmute(Box::new(data)) });
  }

  pub fn load_data<D: Default + 'a>(&mut self, key: impl Into<DepId>) -> &'a mut D {
    let boxed = self
      .data
      .entry(key.into())
      .or_insert_with(|| unsafe { mem::transmute(Box::new(D::default())) });
    unsafe { mem::transmute(boxed.as_mut()) }
  }

  pub fn take_labels(&mut self) -> Option<Rc<Vec<LabelEntity<'a>>>> {
    if self.pending_labels.is_empty() {
      None
    } else {
      Some(Rc::new(mem::take(&mut self.pending_labels)))
    }
  }

  pub fn current_span(&self) -> Span {
    *self.span_stack.last().unwrap()
  }

  pub fn add_diagnostic(&mut self, message: impl Into<String>) {
    let span = self.current_span();
    self
      .tree_shaker
      .0
      .diagnostics
      .borrow_mut()
      .insert(message.into() + format!(" at {}-{}", span.start, span.end).as_str());
  }

  pub fn push_stmt_span(&mut self, node: &'a impl GetSpan, decl: bool) {
    let span = node.span();
    self.span_stack.push(span);
    if !decl {
      if let Some(debugger) = &mut self.logger {
        debugger.push_event(DebuggerEvent::PushStmtSpan(span));
      }
    }
  }

  pub fn push_expr_span(&mut self, node: &'a impl GetSpan) {
    let span = node.span();
    self.span_stack.push(span);
    if let Some(debugger) = &mut self.logger {
      debugger.push_event(DebuggerEvent::PushExprSpan(span));
    }
  }

  pub fn pop_stmt_span(&mut self, decl: bool) {
    self.span_stack.pop();
    if !decl {
      if let Some(debugger) = &mut self.logger {
        debugger.push_event(DebuggerEvent::PopStmtSpan);
      }
    }
  }

  pub fn pop_expr_span(&mut self) {
    self.span_stack.pop();
    if let Some(debugger) = &mut self.logger {
      debugger.push_event(DebuggerEvent::PopExprSpan);
    }
  }
}

impl<'a> Into<&'a Allocator> for Analyzer<'a> {
  fn into(self) -> &'a Allocator {
    self.allocator
  }
}
