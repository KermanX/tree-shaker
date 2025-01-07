mod arguments;
mod array;
mod entity;
mod evaluation;
mod function;
mod primitives;
mod traverse;

pub use arguments::*;
pub use array::*;
pub use entity::*;
pub use evaluation::*;
pub use function::*;
pub use primitives::*;
pub use traverse::*;

pub trait Host<'a>:
  Sized
  + EntityHost<'a>
  + PrimitivesHost<'a>
  + ArrayHost<'a>
  + TraverseHost<'a>
  + EvaluationHost<'a>
  + FunctionHost<'a>
  + ArgumentsHost<'a>
{
}
