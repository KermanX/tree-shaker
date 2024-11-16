use std::borrow::BorrowMut;

use crate::{
  builtins::{constants::OBJECT_CONSTRUCTOR_OBJECT_ID, Builtins},
  entity::{Entity, ObjectEntity, ObjectPropertyValue},
  init_namespace,
};

impl<'a> Builtins<'a> {
  pub fn init_object_constructor(&mut self) {
    let factory = self.factory;

    let object =
      ObjectEntity::new_builtin(OBJECT_CONSTRUCTOR_OBJECT_ID, &self.prototypes.function, false);
    object
      .rest
      .borrow_mut()
      .values
      .push(ObjectPropertyValue::Field(factory.immutable_unknown, Some(true)));

    init_namespace!(object, {
      "prototype" => factory.immutable_unknown,
      "assign" => self.create_object_assign_impl(),
    });

    self.globals.borrow_mut().insert("Object", factory.entity(object));
  }

  fn create_object_assign_impl(&self) -> Entity<'a> {
    self.factory.implemented_builtin_fn(|analyzer, dep, _, args| {
      let (known, rest, deps) = args.iterate(analyzer, dep.cloned());

      if known.len() < 2 {
        return analyzer.factory.computed_unknown((dep, args));
      }

      let target = known[0];

      let mut assign = |source: Entity<'a>, indeterminate: bool| {
        let (properties, deps) = source.enumerate_properties(analyzer, dep.cloned());
        for (definite, key, value) in properties {
          if indeterminate || !definite {
            analyzer.push_indeterminate_cf_scope();
          }
          target.set_property(analyzer, deps.cloned(), key, value);
          if indeterminate || !definite {
            analyzer.pop_cf_scope();
          }
        }
      };

      for source in &known[1..] {
        assign(*source, false);
      }
      if let Some(rest) = rest {
        assign(rest, true);
      }

      analyzer.factory.computed(target, deps)
    })
  }
}
