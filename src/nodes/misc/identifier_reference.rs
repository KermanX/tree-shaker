use crate::ast::AstType2;
use crate::entity::entity::Entity;
use crate::entity::literal::LiteralEntity;
use crate::entity::unknown::UnknownEntity;
use crate::{transformer::Transformer, Analyzer};
use oxc::ast::ast::IdentifierReference;

const AST_TYPE: AstType2 = AstType2::IdentifierReference;

#[derive(Debug, Default, Clone)]
pub struct Data {
  resolvable: bool,
}

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_identifier_reference_read(
    &mut self,
    node: &'a IdentifierReference<'a>,
  ) -> Entity<'a> {
    if node.name == "undefined" {
      return LiteralEntity::new_undefined();
    }

    let reference = self.sematic.symbols().get_reference(node.reference_id().unwrap());
    assert!(reference.is_read());
    let symbol = reference.symbol_id();

    self.set_data(AST_TYPE, node, Data { resolvable: symbol.is_some() });

    if let Some(symbol) = symbol {
      self.get_symbol(&symbol).clone()
    } else {
      // TODO: Handle globals
      self.refer_global_dep();
      UnknownEntity::new_unknown()
    }
  }

  pub(crate) fn exec_identifier_reference_export(&mut self, node: &'a IdentifierReference<'a>) {
    let reference = self.sematic.symbols().get_reference(node.reference_id().unwrap());
    assert!(reference.is_read());
    let symbol = reference.symbol_id();
    self.exports.push(symbol.unwrap());
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_identifier_reference_read(
    &mut self,
    node: IdentifierReference<'a>,
    need_val: bool,
  ) -> Option<IdentifierReference<'a>> {
    let data = self.get_data::<Data>(AST_TYPE, &node);

    (!data.resolvable || need_val).then(|| node)
  }
}
