use crate::analyzer::Analyzer;
use oxc::{ast::AstKind, semantic::ScopeId};

impl<'a> Analyzer<'a> {
  /// Not that this is for flamegraph only. May not conform to the standard.
  pub fn resolve_function_name(&self, scope_id: ScopeId) -> Option<&'a str> {
    let node_id = self.semantic.scopes().get_node_id(scope_id);
    let parent = self.semantic.nodes().parent_kind(node_id)?;
    match parent {
      AstKind::VariableDeclarator(node) => node.id.get_identifier().map(|a| a.as_str()),
      AstKind::AssignmentPattern(node) => node.left.get_identifier().map(|a| a.as_str()),
      AstKind::AssignmentExpression(node) => node.left.get_identifier(),
      AstKind::ObjectProperty(node) => {
        node.key.static_name().map(|s| self.allocator.alloc(s.to_string()).as_str())
      }
      AstKind::ImportSpecifier(node) => Some(node.imported.name().as_str()),
      _ => None,
    }
  }
}
