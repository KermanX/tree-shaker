use oxc::ast::ast::LabelIdentifier;

#[derive(Debug)]
pub(crate) struct LoopScope<'a> {
  pub(crate) label: Option<&'a str>,
  pub(crate) broken: Option<bool>,
  pub(crate) continued: Option<bool>,
}

impl<'a> LoopScope<'a> {
  pub(crate) fn new(label: Option<&'a LabelIdentifier<'a>>) -> Self {
    LoopScope {
      label: label.map(|label| label.name.as_str()),
      broken: Some(false),
      continued: Some(false),
    }
  }
}
