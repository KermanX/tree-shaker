use crate::entity::{Entity, EntityFactory};

pub fn create_react_memo_impl<'a>(factory: &'a EntityFactory<'a>) -> Entity<'a> {
  factory.implemented_builtin_fn(|analyzer, dep, _this, args| {
    let renderer = args.destruct_as_array(analyzer, dep, 1).0[0];

    renderer
  })
}
