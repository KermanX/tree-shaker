use super::{
  arguments::{ArgumentsSource, ArgumentsSourceUnknown},
  EntityValue,
};
use crate::Analyzer;
use std::rc::Rc;

impl EntityValue {
  pub fn get_property(&self, key: &EntityValue) -> Rc<EntityValue> {
    match self {
      EntityValue::Object(obj) => obj.get_property(key),
      EntityValue::Union(keys) => Rc::new(EntityValue::Union(
        keys.iter().map(|key| key.get_property(key)).collect::<Vec<Rc<EntityValue>>>(),
      )),
      EntityValue::Unknown => Rc::new(EntityValue::Unknown),
      _ => unreachable!(),
    }
  }

  pub fn call<'a>(
    &self,
    analyzer: &mut Analyzer<'a>,
    this: EntityValue,
    args: &'a dyn ArgumentsSource<'a>,
  ) -> (bool, EntityValue) {
    match self {
      EntityValue::Function(func) => func.call(analyzer, this, args),
      EntityValue::Union(funcs) => {
        let mut pure = true;
        let mut values = vec![];
        for func in funcs {
          let ret = func.call(analyzer, this.clone(), args);
          pure &= ret.0;
          values.push(Rc::new(ret.1));
        }
        (pure, EntityValue::Union(values).simplify())
      }
      _ => (true, EntityValue::Unknown),
    }
  }

  pub fn consume<'a>(&self, analyzer: &mut Analyzer<'a>) {
    match self {
      EntityValue::Function(func) => {
        func.call(
          analyzer,
          EntityValue::Unknown,
          analyzer.allocator.alloc(ArgumentsSourceUnknown {}),
        );
      }
      EntityValue::Union(values) => {
        for value in values {
          value.consume(analyzer);
        }
      }
      _ => {}
    }
  }
}
