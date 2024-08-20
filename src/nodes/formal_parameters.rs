use oxc::ast::ast::FormalParameters;
use crate::{entity::Entity, TreeShaker};

#[derive(Debug, Default, Clone)]
pub struct Data {}

impl<'a> TreeShaker<'a> {
  pub(crate) fn exec_formal_parameters(&mut self, node: &'a FormalParameters, args: &[Entity]) {
    let data = self.load_data::<Data>(node);
    
  }
}
