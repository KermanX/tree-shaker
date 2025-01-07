use crate::nodes::expr::ExpressionAnalyzer;

pub trait Analyzer<'a>: ExpressionAnalyzer<'a> {
  type Entity;

  fn new_undefined(&self) -> Self::Entity
  where
    Self: Analyzer<'a>;
}

// impl<'a, H: Host<'a>> Analyzer<'a, H> {
//   pub fn new(host: H) -> Self {
//     Analyzer { host, scoping: Scoping::new(factory) }
//   }

//   pub fn exec_program(&mut self, node: &'a Program<'a>) {
//     // Top level is always preserved
//     let top_level_call_id = self.call_scope().call_id;
//     self.refer_dep(top_level_call_id);

//     self.init_statement_vec(AstKind::Program(node), &node.body);

//     self.consume_exports();

//     let mut round = 0usize;
//     loop {
//       round += 1;
//       if round > 1000 {
//         panic!("Possible infinite loop in post analysis");
//       }

//       let mut dirty = false;
//       dirty |= self.consume_top_level_uncaught();
//       dirty |= self.call_exhaustive_callbacks();
//       dirty |= self.post_analyze_handle_conditional();
//       dirty |= self.post_analyze_handle_loops();
//       if !dirty {
//         break;
//       }
//     }

//     self.scope_context.assert_final_state();

//     // println!("debug: {:?}", self.debug);

//     #[cfg(feature = "flame")]
//     flamescope::dump(&mut std::fs::File::create("flamescope.json").unwrap()).unwrap();
//   }

//   pub fn consume_exports(&mut self) {
//     if let Some(entity) = self.default_export.take() {
//       entity.consume(self)
//     }
//     for symbol in self.named_exports.clone() {
//       let entity = self.read_symbol(symbol).unwrap();
//       entity.consume(self);
//     }
//   }

//   pub fn consume_top_level_uncaught(&mut self) -> bool {
//     let thrown_values = &mut self.call_scope_mut().try_scopes.last_mut().unwrap().thrown_values;
//     if thrown_values.is_empty() {
//       false
//     } else {
//       let values = mem::take(thrown_values);
//       self.consume(values);
//       true
//     }
//   }
// }
