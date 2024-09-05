use crate::{analyzer::Analyzer, transformer::Transformer};
use oxc::ast::ast::ClassElement;

impl<'a> Analyzer<'a> {
  pub fn exec_class_element(&mut self, node: &'a ClassElement<'a>) {
    match node {
      ClassElement::StaticBlock(node) => self.exec_static_block(node),
      ClassElement::MethodDefinition(node) => self.exec_method_definition(node),
      ClassElement::PropertyDefinition(node) => self.exec_property_definition(node),
      ClassElement::AccessorProperty(node) => self.exec_accessor_property(node),
      ClassElement::TSIndexSignature(_node) => unreachable!(),
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_class_element(&self, node: &'a ClassElement<'a>) -> ClassElement<'a> {
    match node {
      ClassElement::StaticBlock(node) => self.transform_static_block(node),
      ClassElement::MethodDefinition(node) => self.transform_method_definition(node),
      ClassElement::PropertyDefinition(node) => self.transform_property_definition(node),
      ClassElement::AccessorProperty(node) => self.transform_accessor_property(node),
      ClassElement::TSIndexSignature(_node) => unreachable!(),
    }
  }
}
