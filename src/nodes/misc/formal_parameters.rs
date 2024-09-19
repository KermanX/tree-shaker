use crate::{
  analyzer::Analyzer,
  ast::{AstType2, DeclarationKind},
  entity::entity::Entity,
  transformer::Transformer,
};
use oxc::ast::ast::{FormalParameter, FormalParameters};

const AST_TYPE: AstType2 = AstType2::FormalParameter;

#[derive(Debug, Default)]
pub struct Data<'a> {
  elements_init: Vec<Vec<Entity<'a>>>,
  rest_init: Vec<Entity<'a>>,
}

impl<'a> Analyzer<'a> {
  pub fn exec_formal_parameters(
    &mut self,
    node: &'a FormalParameters<'a>,
    args: Entity<'a>,
    kind: DeclarationKind,
  ) {
    let (elements_init, rest_init) = args.destruct_as_array(self, (), node.items.len());

    let data = self.load_data::<Data>(AST_TYPE, node);
    data.elements_init.push(elements_init.clone());
    data.rest_init.push(rest_init.clone());

    for (param, _) in node.items.iter().zip(&elements_init) {
      self.declare_binding_pattern(&param.pattern, false, kind);
    }

    for (param, init) in node.items.iter().zip(elements_init) {
      self.init_binding_pattern(&param.pattern, Some(init));
    }

    if let Some(rest) = &node.rest {
      self.declare_binding_rest_element(rest, false, kind);
      self.init_binding_rest_element(rest, rest_init);
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_formal_parameters(
    &self,
    node: &'a FormalParameters<'a>,
  ) -> FormalParameters<'a> {
    let data = self.get_data::<Data>(AST_TYPE, node);

    let FormalParameters { span, items, rest, kind, .. } = node;

    let mut transformed_items = self.ast_builder.vec();

    for (index, param) in items.iter().enumerate() {
      let FormalParameter { span, decorators, pattern, .. } = param;

      let pattern = self.transform_binding_pattern(pattern, false);

      if pattern.is_some() {
        for dep in &data.elements_init {
          dep[index].refer_dep_shallow(self);
        }
      }

      transformed_items.push(self.ast_builder.formal_parameter(
        *span,
        self.clone_node(decorators),
        pattern.unwrap_or_else(|| self.build_unused_binding_pattern(*span)),
        None,
        false,
        false,
      ));
    }

    let transformed_rest = match rest {
      Some(rest) => self.transform_binding_rest_element(rest, false),
      None => None,
    };

    self.ast_builder.formal_parameters(*span, *kind, transformed_items, transformed_rest)
  }
}
