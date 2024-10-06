use crate::scope::CfScopeKind;
use oxc::{index::Idx, semantic::ScopeId, span::Span};
use rustc_hash::FxHashMap;
use std::cell::RefCell;

#[derive(Debug)]
pub enum DebuggerEvent {
  PushStmtSpan(Span),
  PopStmtSpan,
  PushExprSpan(Span),
  PopExprSpan,
  PushCallScope(Span, Vec<ScopeId>, usize, ScopeId),
  PopCallScope,
  PushCfScope(ScopeId, CfScopeKind, Option<bool>),
  UpdateCfScopeExited(ScopeId, Option<bool>),
  PopCfScope,
  ReplaceVarScopeStack(Vec<ScopeId>),
  PushVarScope(ScopeId, ScopeId),
  PopVarScope,
}

#[derive(Debug, Default)]
pub struct Logger {
  events: RefCell<Vec<DebuggerEvent>>,
  fn_calls: RefCell<FxHashMap<Span, (String, usize)>>,
}

impl Logger {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn push_event(&self, event: DebuggerEvent) {
    self.events.borrow_mut().push(event);
  }

  pub fn push_fn_call(&self, span: Span, name: String) {
    self.fn_calls.borrow_mut().entry(span).or_insert((name, 0)).1 += 1;
  }

  pub fn serialize(&self) -> Vec<String> {
    let mut logs = Vec::new();
    for event in self.events.borrow().iter() {
      logs.push(Self::serialize_event(event));
    }
    logs
  }

  fn serialize_event(event: &DebuggerEvent) -> String {
    match event {
      DebuggerEvent::PushStmtSpan(span) => format!("PushStmtSpan {}", Self::serialize_span(span)),
      DebuggerEvent::PopStmtSpan => "PopStmtSpan".to_string(),
      DebuggerEvent::PushExprSpan(span) => format!("PushExprSpan {}", Self::serialize_span(span)),
      DebuggerEvent::PopExprSpan => "PopExprSpan".to_string(),
      DebuggerEvent::PushCallScope(
        span,
        old_variable_scope_stack,
        cf_scope_depth,
        body_variable_scope,
      ) => {
        format!(
          "PushCallScope {} {} {} {}",
          Self::serialize_span(span),
          old_variable_scope_stack
            .iter()
            .map(Self::serialize_scope_id)
            .collect::<Vec<_>>()
            .join(","),
          cf_scope_depth,
          Self::serialize_scope_id(body_variable_scope),
        )
      }
      DebuggerEvent::PopCallScope => "PopCallScope".to_string(),
      DebuggerEvent::PushCfScope(scope_id, kind, exited) => {
        format!(
          "PushCfScope {} {} {}",
          Self::serialize_scope_id(scope_id),
          match kind {
            CfScopeKind::BreakableWithoutLabel => "Breakable",
            CfScopeKind::ConditionalExpression => "CondExpr",
            CfScopeKind::Exhaustive => "Exhaustive",
            CfScopeKind::IfStatement => "IfStmt",
            CfScopeKind::Normal => "Normal",
            CfScopeKind::Function => "Function",
            CfScopeKind::Module => "Module",
            CfScopeKind::Continuable => "Continuable",
            CfScopeKind::LogicalExpression => "LogicalExpr",
          },
          Self::serialize_exited(exited),
        )
      }
      DebuggerEvent::UpdateCfScopeExited(scope_id, exited) => {
        format!(
          "UpdateCfScopeExited {} {}",
          Self::serialize_scope_id(scope_id),
          Self::serialize_exited(exited)
        )
      }
      DebuggerEvent::PopCfScope => "PopCfScope".to_string(),
      DebuggerEvent::ReplaceVarScopeStack(scope_ids) => {
        format!(
          "ReplaceVarScopeStack {}",
          scope_ids.iter().map(Self::serialize_scope_id).collect::<Vec<_>>().join(",")
        )
      }
      DebuggerEvent::PushVarScope(scope_id, cf_scope_id) => {
        format!(
          "PushVarScope {} {}",
          Self::serialize_scope_id(scope_id),
          Self::serialize_scope_id(cf_scope_id)
        )
      }
      DebuggerEvent::PopVarScope => "PopVarScope".to_string(),
    }
  }

  fn serialize_span(span: &Span) -> String {
    format!("{}-{}", span.start, span.end)
  }

  fn serialize_scope_id(scope_id: &ScopeId) -> String {
    scope_id.index().to_string()
  }

  fn serialize_exited(exited: &Option<bool>) -> String {
    match exited {
      Some(true) => "true".to_string(),
      Some(false) => "false".to_string(),
      None => "unknown".to_string(),
    }
  }

  pub fn print_fn_calls(&self) {
    for (span, (name, count)) in self.fn_calls.borrow().iter() {
      println!("{}-{} {} x{}", span.start, span.end, name, count);
    }
  }
}
