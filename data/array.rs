

fn length_of_array_like(this: Var) -> Var {
  js_var!(+this["length"])
}

fn array_prototype_at(this: Var, index: Var) -> Var {
  let len = length_of_array_like(this);
  let relative_index = js_var!(+index);
  let k = if js_bool!(index_num >= 0) { relative_index } else { js_var!(len + relative_index) };
  if js_bool!(k < 0) || js_bool!(k >= len) {
    js_var!(undefined)
  } else {
    js_var!(this[k])
  }
}

fn array_prototype_map(this: Var, callback: Var, thisarg: Var) -> Var {
  let len = length_of_array_like(this);
  if !is_callable(callback) {
    return js_throw!("TypeError", "callback is not a function");
  }
  let A = array_species_create(this, len);
  let k = 0;
  while k < len {
    let Pk = to_string(k);
    let kPresent = has_property(this, Pk);
    if kPresent {
      let kValue = get_property(this, Pk);
      let mappedValue = call!()
    }
  }
}