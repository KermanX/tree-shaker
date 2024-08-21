use crate::{
  symbol::{arguments::ArgumentsEntity, SymbolSource},
  Analyzer,
};

use super::Entity;
use std::rc::Rc;

impl Entity {
  pub fn get_property(&self, key: &Entity) -> Rc<Entity> {
    match self {
      Entity::Object(obj) => obj.get_property(key),
      Entity::Union(keys) => Rc::new(Entity::Union(
        keys.iter().map(|key| key.get_property(key)).collect::<Vec<Rc<Entity>>>(),
      )),
      Entity::Unknown => Rc::new(Entity::Unknown),
      _ => unreachable!(),
    }
  }

  pub fn call<'a>(
    &self,
    analyzer: &mut Analyzer<'a>,
    this: Entity,
    args: ArgumentsEntity<'a>,
  ) -> (bool, Entity) {
    match self {
      Entity::Function(func) => func.call(analyzer, this, args),
      Entity::Union(funcs) => {
        let mut pure = true;
        let mut values = vec![];
        for func in funcs {
          let ret = func.call(analyzer, this.clone(), args.clone());
          pure &= ret.0;
          values.push(Rc::new(ret.1));
        }
        (pure, Entity::Union(values).simplify())
      }
      _ => (true, Entity::Unknown),
    }
  }

  pub fn consume<'a>(&self, analyzer: &mut Analyzer<'a>) {
    match self {
      Entity::Function(func) => {
        func.call(
          analyzer,
          Entity::Unknown,
          ArgumentsEntity::new(vec![(true, SymbolSource::Unknown)]),
        );
      }
      Entity::Union(funcs) => {
        for func in funcs {
          func.consume(analyzer);
        }
      }
      _ => {}
    }
  }
}
