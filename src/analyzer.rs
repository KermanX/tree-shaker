use crate::{
  ast::AstType2,
  builtins::Builtins,
  data::{get_node_ptr, Diagnostics, ExtraData, ReferredNodes, StatementVecData, VarDeclarations},
  entity::{Entity, EntityOpHost, LabelEntity},
  scope::{exhaustive::TrackerRunner, ScopeContext},
  TreeShakeConfig,
};
use oxc::{
  allocator::Allocator,
  ast::ast::Program,
  semantic::{Semantic, SymbolId},
  span::{GetSpan, Span},
};
use rustc_hash::{FxHashMap, FxHashSet};
use std::mem;

pub struct Analyzer<'a> {
  pub config: TreeShakeConfig,
  pub allocator: &'a Allocator,
  pub semantic: Semantic<'a>,
  pub diagnostics: &'a mut Diagnostics,
  pub current_span: Vec<Span>,
  pub data: ExtraData<'a>,
  pub referred_nodes: ReferredNodes<'a>,
  pub var_decls: VarDeclarations<'a>,
  pub named_exports: Vec<SymbolId>,
  pub default_export: Option<Entity<'a>>,
  pub exhaustive_deps: FxHashMap<SymbolId, FxHashSet<TrackerRunner<'a>>>,
  pub scope_context: ScopeContext<'a>,
  pub pending_labels: Vec<LabelEntity<'a>>,
  pub builtins: Builtins<'a>,
  pub entity_op: EntityOpHost<'a>,
}

impl<'a> Analyzer<'a> {
  pub fn new(
    config: TreeShakeConfig,
    allocator: &'a Allocator,
    semantic: Semantic<'a>,
    diagnostics: &'a mut Diagnostics,
  ) -> Self {
    Analyzer {
      config,
      allocator,
      semantic,
      diagnostics,
      current_span: vec![],
      data: Default::default(),
      referred_nodes: Default::default(),
      var_decls: Default::default(),
      named_exports: Vec::new(),
      default_export: None,
      exhaustive_deps: Default::default(),
      scope_context: ScopeContext::new(),
      pending_labels: Vec::new(),
      builtins: Builtins::new(),
      entity_op: EntityOpHost::new(allocator),
    }
  }

  pub fn exec_program(&mut self, node: &'a Program<'a>) {
    let data = self.load_data::<StatementVecData>(AstType2::Program, node);
    self.exec_statement_vec(data, &node.body);

    // Consume exports
    self.default_export.take().map(|entity| entity.consume(self));
    for symbol in self.named_exports.clone() {
      let entity = self.read_symbol(symbol).unwrap();
      entity.consume(self);
    }
    // Consume uncaught thrown values
    self.call_scope_mut().try_scopes.pop().unwrap().thrown_val().map(|entity| {
      entity.consume(self);
    });

    self.scope_context.assert_final_state();
  }
}

impl<'a> Analyzer<'a> {
  pub fn set_data<T>(&mut self, ast_type: AstType2, node: &'a T, data: impl Default + 'a) {
    let key = (ast_type, get_node_ptr(node));
    self.data.insert(key, unsafe { mem::transmute(Box::new(data)) });
  }

  pub fn load_data<D: Default + 'a>(
    &mut self,
    ast_type: AstType2,
    node: &'a impl GetSpan,
  ) -> &'a mut D {
    let key = (ast_type, get_node_ptr(node));
    let boxed =
      self.data.entry(key).or_insert_with(|| unsafe { mem::transmute(Box::new(D::default())) });
    unsafe { mem::transmute(boxed.as_mut()) }
  }

  pub fn add_diagnostic(&mut self, message: impl Into<String>) {
    let span = self.current_span.last().unwrap();
    self.diagnostics.insert(message.into() + format!(" at {}-{}", span.start, span.end).as_str());
  }
}
