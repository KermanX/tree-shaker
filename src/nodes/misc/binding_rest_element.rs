use crate::ast::AstType2;
use crate::entity::entity::Entity;
use crate::entity::object::ObjectEntity;
use crate::{transformer::Transformer, Analyzer};
use oxc::ast::ast::{BindingRestElement, PropertyKind};
use oxc::span::GetSpan;

const AST_TYPE: AstType2 = AstType2::BindingRestElement;

#[derive(Debug, Default)]
struct Data {
  has_effect: bool,
}

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_binding_rest_element(
    &mut self,
    node: &'a BindingRestElement<'a>,
    init: Entity<'a>,
    exporting: bool,
  ) {
    let (has_effect, properties) = init.enumerate_properties(self);

    let object = ObjectEntity::new_empty_object();
    for (key, value) in properties {
      object.init_property(PropertyKind::Init, key, value);
    }

    self.exec_binding_pattern(&node.argument, (false, init), exporting);

    let data = self.load_data::<Data>(AST_TYPE, node);
    data.has_effect |= has_effect;
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_binding_rest_element(
    &mut self,
    node: BindingRestElement<'a>,
  ) -> Option<BindingRestElement<'a>> {
    let data = self.get_data::<Data>(AST_TYPE, &node);

    let BindingRestElement { span, argument, .. } = node;
    let argument_span = argument.span();

    let argument = self.transform_binding_pattern(argument);

    if let Some(argument) = argument {
      Some(self.ast_builder.binding_rest_element(span, argument))
    } else if data.has_effect {
      Some(
        self
          .ast_builder
          .binding_rest_element(span, self.build_unused_binding_pattern(argument_span)),
      )
    } else {
      None
    }
  }
}
