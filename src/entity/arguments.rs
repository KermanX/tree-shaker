use super::{
  array::{ArrayEntity, UNKNOWN_ARRAY},
  Entity,
};

#[derive(Clone)]
pub struct ArgumentsEntity {
  args: Vec<(bool, Entity)>,
}

impl ArgumentsEntity {
  pub(crate) fn new(args: Vec<(bool, Entity)>) -> Self {
    ArgumentsEntity { args }
  }

  pub(crate) fn resolve(&self, length: usize) -> (Vec<Entity>, Entity) {
    // TODO: Properly handle rest
    let mut resolved = vec![];
    let mut is_rest_unknown = false;
    for (expend, arg) in &self.args {
      if *expend {
        if let Some(tuple) = arg.to_array().as_tuple() {
          for arg in tuple {
            resolved.push(arg.clone());
          }
        } else {
          if resolved.len() < length {
            for _ in resolved.len()..length {
              resolved.push(Entity::Unknown);
            }
            is_rest_unknown = true;
          }
          break;
        }
      } else {
        resolved.push(arg.clone());
      }
    }
    let (args, rest) = resolved.split_at(length);
    let rest = Entity::Array(if is_rest_unknown {
      UNKNOWN_ARRAY.clone()
    } else {
      ArrayEntity::from_tuple(rest)
    });

    assert!(args.len() == length);

    (args.to_vec(), rest)
  }
}
