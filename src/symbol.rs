use crate::{
  entity::{arguments::ArgumentsEntity, function::FunctionEntity, Entity},
  TreeShaker,
};
use oxc::{
  ast::ast::{
    AssignmentExpression, BindingRestElement, Class, FormalParameter, Function, UsingDeclaration,
    VariableDeclarator,
  },
  semantic::SymbolId,
};

#[derive(Debug, Clone)]
pub enum SymbolSource<'a> {
  VariableDeclarator(&'a VariableDeclarator<'a>),
  Function(&'a Function<'a>),
  ClassDeclaration(&'a Class<'a>),
  UsingDeclaration(&'a UsingDeclaration<'a>),
  FormalParameter(&'a FormalParameter<'a>, Entity),
  BindingRestElement(&'a BindingRestElement<'a>, Entity),
  AssignmentExpression(&'a AssignmentExpression<'a>),
}

impl<'a> TreeShaker<'a> {
  pub(crate) fn declare_symbol(&mut self, source: SymbolSource<'a>, symbol: SymbolId) {
    self.symbol_source.insert(symbol, source);
  }

  pub(crate) fn read_symbol(&mut self, symbol: SymbolId) -> Entity {
    let source = self.symbol_source.get(&symbol).expect("Missing declaration");

    match source {
      SymbolSource::VariableDeclarator(node) => self.refer_variable_declarator(node, symbol),
      SymbolSource::Function(node) => Entity::Function(FunctionEntity::new(node.span)),
      _ => todo!(),
    }
  }

  pub(crate) fn call_symbol(
    &mut self,
    symbol: SymbolId,
    this: Entity,
    args: ArgumentsEntity,
  ) -> (bool, Entity) {
    let source = self.symbol_source.get(&symbol).expect("Missing declaration");

    match source {
      SymbolSource::Function(node) => self.call_function(node, this, args),
      _ => (true, Entity::Unknown),
    }
  }
}
