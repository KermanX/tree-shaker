use crate::{host::Host, 
  ast::{AstKind2, DeclarationKind},
  dep::DepId,
  entity::Entity,
    Analyzer,
};
use oxc::{
  ast::{
    ast::{
      ArrayPattern, AssignmentPattern, BindingPattern, BindingPatternKind, BindingProperty,
      ObjectPattern,
    },
    NONE,
  },
  span::GetSpan,
};

#[derive(Debug, Default)]
struct ObjectPatternData {
  need_destruct: bool,
}

#[derive(Debug, Default)]
struct AssignmentPatternData {
  need_right: bool,
}

impl<'a, H: Host<'a>> Analyzer<'a, H> {
  pub fn declare_binding_pattern(
    &mut self,
    node: &'a BindingPattern<'a>,
    exporting: bool,
    kind: DeclarationKind,
  ) {
    match &node.kind {
      BindingPatternKind::BindingIdentifier(node) => {
        self.declare_binding_identifier(node, exporting, kind);
      }
      BindingPatternKind::ObjectPattern(node) => {
        for property in &node.properties {
          self.declare_binding_pattern(&property.value, exporting, kind);
        }
        if let Some(rest) = &node.rest {
          self.declare_binding_rest_element(rest, exporting, kind);
        }
      }
      BindingPatternKind::ArrayPattern(node) => {
        for element in node.elements.iter().flatten() {
          self.declare_binding_pattern(element, exporting, kind);
        }
        if let Some(rest) = &node.rest {
          self.declare_binding_rest_element(rest, exporting, kind);
        }
      }
      BindingPatternKind::AssignmentPattern(node) => {
        self.declare_binding_pattern(&node.left, exporting, kind);
      }
    }
  }

  pub fn init_binding_pattern(&mut self, node: &'a BindingPattern<'a>, init: Option<H::Entity>) {
    match &node.kind {
      BindingPatternKind::BindingIdentifier(node) => {
        self.init_binding_identifier(node, init);
      }
      BindingPatternKind::ObjectPattern(node) => {
        let init = init.unwrap_or_else(|| {
          self.thrown_builtin_error("Missing initializer in destructuring declaration");
          self.factory.unknown()
        });

        let is_nullish = init.test_nullish();
        if is_nullish != Some(false) {
          if is_nullish == Some(true) {
            self.thrown_builtin_error("Cannot destructure nullish value");
          } else {
            self.may_throw();
          }
          init.consume(self);
                   data.need_destruct = true;
        }

        let mut enumerated = vec![];
        for property in &node.properties {
          let dep = self.consumable(DepId::from(AstKind2::BindingProperty(property)));

          self.push_dependent_cf_scope(init);
          let key = self.exec_property_key(&property.key);
          self.pop_cf_scope();

          enumerated.push(key);
          let init = init.get_property(self, dep, key);
          self.init_binding_pattern(&property.value, Some(init));
        }
        if let Some(rest) = &node.rest {
          let dep = AstKind2::BindingRestElement(rest.as_ref());
          let init = self.exec_object_rest(dep, init, enumerated);
          self.init_binding_rest_element(rest, init);
        }
      }
      BindingPatternKind::ArrayPattern(node) => {
        let init = init.unwrap_or_else(|| {
          self.thrown_builtin_error("Missing initializer in destructuring declaration");
          self.factory.unknown()
        });

        let (element_values, rest_value, dep) = init.destruct_as_array(
          self,
          self.consumable(AstKind2::ArrayPattern(node)),
          node.elements.len(),
          node.rest.is_some(),
        );

        self.push_dependent_cf_scope(dep);
        for (element, value) in node.elements.iter().zip(element_values) {
          if let Some(element) = element {
            self.init_binding_pattern(element, Some(value));
          }
        }
        if let Some(rest) = &node.rest {
          self.init_binding_rest_element(rest, rest_value.unwrap());
        }
        self.pop_cf_scope();
      }
      BindingPatternKind::AssignmentPattern(node) => {
        let (need_right, binding_val) = self.exec_with_default(&node.right, init.unwrap());

        let data =
          self.load_data::<AssignmentPatternData>(AstKind2::AssignmentPattern(node.as_ref()));
        data.need_right |= need_right;

        self.init_binding_pattern(&node.left, Some(binding_val));
      }
    }
  }
}

