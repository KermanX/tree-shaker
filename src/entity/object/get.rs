use super::ObjectEntity;
use crate::{
  analyzer::Analyzer,
  consumable::Consumable,
  entity::{consumed_object, Entity, LiteralEntity},
};

impl<'a> ObjectEntity<'a> {
  pub fn get_property(
    &self,
    rc: Entity<'a>,
    analyzer: &mut Analyzer<'a>,
    dep: Consumable<'a>,
    key: Entity<'a>,
  ) -> Entity<'a> {
    if self.consumed.get() {
      return consumed_object::get_property(rc, analyzer, dep, key);
    }

    analyzer.mark_object_property_exhaustive_read(self.cf_scope, self.object_id);

    let mut mangable = false;
    let mut values = vec![];
    let mut getters = vec![];
    let mut non_existent = vec![];

    let mut check_rest = false;
    let mut may_add_undefined = false;
    let key = key.get_to_property_key(analyzer);
    if let Some(key_literals) = key.get_to_literals(analyzer) {
      mangable = self.check_mangable(analyzer, &key_literals);
      let mut string_keyed = self.string_keyed.borrow_mut();
      for key_literal in key_literals {
        match key_literal {
          LiteralEntity::String(key_str, key_atom) => {
            if let Some(property) = string_keyed.get_mut(key_str) {
              if mangable {
                property.get_mangable(
                  analyzer,
                  &mut values,
                  &mut getters,
                  &mut non_existent,
                  key,
                  key_atom.unwrap(),
                );
              } else {
                property.get(analyzer, &mut values, &mut getters, &mut non_existent);
              }
            } else {
              check_rest = true;
              if let Some(val) = self.prototype.get_string_keyed(key_str) {
                values.push(if mangable { analyzer.factory.computed(val, key_atom) } else { val });
              } else {
                may_add_undefined = true;
              }
            }
          }
          LiteralEntity::Symbol(_, _) => todo!(),
          _ => unreachable!("Invalid property key"),
        }
      }

      check_rest |= !non_existent.is_empty();
      may_add_undefined |= !non_existent.is_empty();
    } else {
      self.disable_mangling(analyzer);

      for property in self.string_keyed.borrow_mut().values_mut() {
        property.get(analyzer, &mut values, &mut getters, &mut non_existent);
      }

      // TODO: prototype? Use a config IMO
      // Either:
      // - Skip prototype
      // - Return unknown and call all getters

      check_rest = true;
      may_add_undefined = true;
    }

    if check_rest {
      let mut rest = self.rest.borrow_mut();
      if let Some(rest) = &mut *rest {
        rest.get(analyzer, &mut values, &mut getters, &mut non_existent);
      } else if may_add_undefined {
        values.push(analyzer.factory.undefined);
      }
    }

    let indeterminate_getter = !values.is_empty() || getters.len() > 1 || !non_existent.is_empty();

    {
      let mut unknown_keyed = self.unknown_keyed.borrow_mut();
      unknown_keyed.get(analyzer, &mut values, &mut getters, &mut non_existent);
    }

    if !getters.is_empty() {
      if indeterminate_getter {
        analyzer.push_indeterminate_cf_scope();
      }
      for getter in getters {
        // TODO: Support mangling
        values.push(getter.call_as_getter(analyzer, analyzer.consumable((dep, key)), rc));
      }
      if indeterminate_getter {
        analyzer.pop_cf_scope();
      }
    }

    let value = analyzer.factory.try_union(values).unwrap_or(analyzer.factory.undefined);
    if mangable {
      analyzer.factory.computed(value, (non_existent, dep))
    } else {
      analyzer.factory.computed(value, (non_existent, dep, key))
    }
  }
}
