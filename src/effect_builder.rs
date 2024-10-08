#[macro_export]
macro_rules! build_effect {
  ($builder:expr, $span:expr, $($x:expr),+ $(,)?) => {
    {
      let mut exprs = $builder.vec();
      $($x.map(|e| exprs.push(e));)*
      if exprs.is_empty() {
        None
      }
      else if exprs.len() == 1 {
        exprs.pop()
      }
      else {
        Some($builder.expression_sequence($span, exprs))
      }
    }
  };
  ($builder:expr, $span:expr, $($x:expr),+ $(,)?; $val:expr) => {
    {
      let mut exprs = $builder.vec();
      $($x.map(|e| exprs.push(e));)*
      if exprs.is_empty() {
        $val
      }
      else {
        exprs.push($val);
        $builder.expression_sequence($span, exprs)
      }
    }
  };
}

#[macro_export]
macro_rules! build_effect_from_arr {
  ($builder:expr, $span:expr, $($x:expr),+ $(,)?) => {
    {
      let mut exprs = $builder.vec();
      $(for x in $x { x.map(|e| exprs.push(e)); })*
      if exprs.is_empty() {
        None
      }
      else if exprs.len() == 1 {
        exprs.pop()
      }
      else {
        Some($builder.expression_sequence($span, exprs))
      }
    }
  };
  ($builder:expr, $span:expr, $($x:expr),+ $(,)?; $val:expr) => {
    {
      let mut exprs = $builder.vec();
      $(for x in $x { x.map(|e| exprs.push(e)); })*
      if exprs.is_empty() {
        $val
      }
      else {
        exprs.push($val);
        $builder.expression_sequence($span, exprs)
      }
    }
  };
}
