use crate::ast::AstType2;
use crate::{
  entity::{function::FunctionEntity, EntityValue},
  symbol::{arguments::ArgumentsSource, SymbolSource},
  transformer::Transformer,
  Analyzer,
};
use oxc::ast::ast::Function;

const AST_TYPE: AstType2 = AstType2::Function;

#[derive(Debug, Default, Clone)]
pub struct Data {
  referred: bool,
}

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_function(&mut self, node: &'a Function) -> (bool, EntityValue) {
    if let Some(id) = &node.id {
      self.declare_symbol(SymbolSource::Function(node), id.symbol_id.get().unwrap());
    }

    self.set_data(AST_TYPE, node, Data { referred: self.exporting });

    (false, EntityValue::Function(FunctionEntity::new(node.span)))
  }

  pub(crate) fn calc_function(&self, node: &'a Function<'a>) -> EntityValue {
    EntityValue::Function(FunctionEntity::new(node.span))
  }

  pub(crate) fn refer_function(&mut self, node: &'a Function<'a>) {
    let data = self.load_data::<Data>(AST_TYPE, node);
    data.referred = true;
  }

  pub(crate) fn call_function(
    &mut self,
    node: &'a Function<'a>,
    this: EntityValue,
    args: &'a dyn ArgumentsSource<'a>,
  ) -> (bool, EntityValue) {
    self.exec_formal_parameters(&node.params, args);

    let mut has_effect = false;

    if let Some(body) = &node.body {
      for statement in &body.statements {
        has_effect |= self.exec_statement(statement);
      }
    }

    (has_effect, todo!())
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_function(&self, node: Function<'a>) -> Option<Function<'a>> {
    let data = self.get_data::<Data>(AST_TYPE, &node);

    if !data.referred {
      return None;
    }

    todo!()
  }
}
