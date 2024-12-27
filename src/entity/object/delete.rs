use super::ObjectEntity;
use crate::{
  analyzer::Analyzer,
  consumable::{box_consumable, Consumable},
  entity::{consumed_object, Entity, EntityTrait, LiteralEntity},
  mangling::MangleConstraint,
};

impl<'a> ObjectEntity<'a> {
  pub fn delete_property(&self, analyzer: &mut Analyzer<'a>, dep: Consumable<'a>, key: Entity<'a>) {
    if self.consumed.get() {
      return consumed_object::delete_property(analyzer, dep, key);
    }

    let (has_exhaustive, indeterminate, exec_deps) =
      analyzer.pre_mutate_object(self.cf_scope, self.object_id);

    if has_exhaustive {
      self.consume(analyzer);
      return consumed_object::delete_property(analyzer, dep, key);
    }

    let key = key.get_to_property_key(analyzer);

    {
      let mut unknown_keyed = self.unknown_keyed.borrow_mut();
      if !unknown_keyed.possible_values.is_empty() {
        unknown_keyed.delete(true, box_consumable((exec_deps.clone(), dep.cloned(), key)));
      }
    }

    if let Some(key_literals) = key.get_to_literals(analyzer) {
      let indeterminate = indeterminate || key_literals.len() > 1;
      let mangable = self.check_mangable(analyzer, &key_literals);
      let dep = if mangable {
        box_consumable((exec_deps, dep))
      } else {
        box_consumable((exec_deps, dep, key))
      };

      let mut string_keyed = self.string_keyed.borrow_mut();
      let mut rest = self.rest.borrow_mut();
      for key_literal in key_literals {
        match key_literal {
          LiteralEntity::String(key_str, key_atom) => {
            if let Some(property) = string_keyed.get_mut(key_str) {
              property.delete(
                indeterminate,
                if mangable {
                  let (prev_key, prev_atom) = property.mangling.unwrap();
                  box_consumable((
                    dep.cloned(),
                    // This is a hack
                    analyzer.factory.mangable(
                      analyzer.factory.immutable_unknown,
                      (prev_key, key),
                      analyzer.factory.alloc(MangleConstraint::Eq(prev_atom, key_atom.unwrap())),
                    ),
                  ))
                } else {
                  dep.cloned()
                },
              );
            } else if let Some(rest) = &mut *rest {
              rest.delete(true, box_consumable((dep.cloned(), key)));
            } else if mangable {
              self.add_to_mangling_group(analyzer, key_atom.unwrap());
            }
          }
          LiteralEntity::Symbol(_, _) => todo!(),
          _ => unreachable!("Invalid property key"),
        }
      }
    } else {
      self.disable_mangling(analyzer);

      let dep = box_consumable((exec_deps, dep, key));

      let mut string_keyed = self.string_keyed.borrow_mut();
      for property in string_keyed.values_mut() {
        property.delete(true, dep.cloned());
      }
    }
  }
}