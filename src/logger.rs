use std::cell::RefCell;

use oxc::span::Span;

#[derive(Debug)]
pub enum DebuggerEvent {
  PushStmtSpan(Span),
  PopStmtSpan,
  PushExprSpan(Span),
  PopExprSpan,
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
    }
  }

  fn serialize_span(span: &Span) -> String {
    format!("{}-{}", span.start, span.end)
  }
}
