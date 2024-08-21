pub(crate) struct DataPlaceholder<'a> {
  _phantom: std::marker::PhantomData<&'a ()>,
}
