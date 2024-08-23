use crate::entity::EntityValue;
use std::{cell::LazyCell, rc::Rc};

pub const GLOBAL_THIS: LazyCell<Rc<EntityValue>> = LazyCell::new(|| Rc::new(EntityValue::Unknown));
