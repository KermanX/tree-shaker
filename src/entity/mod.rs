mod arguments;
mod array;
mod builtin_fn;
mod collected;
mod collector;
mod computed;
mod consumed_object;
mod entity;
mod factory;
mod function;
mod label;
mod literal;
mod object;
mod operations;
mod promise;
mod symbol;
mod typeof_result;
mod union;
mod unknown;
mod utils;

pub use array::ArrayEntity;
pub use builtin_fn::PureBuiltinFnEntity;
pub use collector::LiteralCollector;
pub use entity::{Entity, EntityTrait};
pub use factory::EntityFactory;
pub use function::FunctionEntitySource;
pub use label::LabelEntity;
pub use literal::LiteralEntity;
pub use object::{ObjectEntity, ObjectProperty, ObjectPropertyValue};
pub use operations::EntityOpHost;
pub use typeof_result::TypeofResult;
pub use unknown::UnknownEntity;

pub const UNDEFINED_ENTITY: Entity<'static> = Entity(&LiteralEntity::Undefined);
