use oxc::{index::Idx, semantic::ScopeId, span::Span};
use std::cell::RefCell;

#[derive(Debug)]
pub enum DebuggerEvent {
  PushStmtSpan(Span),
  PopStmtSpan,
  PushExprSpan(Span),
  PopExprSpan,
  PushCallScope(Span, Vec<ScopeId>, usize, ScopeId),
  PopCallScope,
}

pub struct Logger {
  events: RefCell<Vec<DebuggerEvent>>,
}

impl Logger {
  pub fn new() -> Self {
    Logger { events: Default::default() }
  }

  pub fn push_event(&self, event: DebuggerEvent) {
    self.events.borrow_mut().push(event);
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
    }
  }

  fn serialize_span(span: &Span) -> String {
    format!("{}-{}", span.start, span.end)
  }

  fn serialize_scope_id(scope_id: &ScopeId) -> String {
    scope_id.index().to_string()
  }
}
