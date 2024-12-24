export function plain_object(aaa, bbb) {
  const obj = {
    foo: aaa,
    bar: bbb,
  };
  console.log(obj.foo);
  console.log(obj.bar);
}

export function computed_property(unknown, aaa, bbb) {
  const key = unknown ? 'foo' : 'bar';
  const obj = {
    foo: aaa,
    bar: bbb
  };
  console.log(obj[key]);
}

export function property_via_destructuring(aaa, bbb) {
  const { foo, bar } = {
    foo: aaa,
    bar: bbb
  };
  console.log(foo);
  console.log(bar);
}

export function with_rest(aaa, bbb) {
  const { foo, ...rest } = {
    foo: aaa,
    bar: bbb
  };
  console.log(rest.bar);
}

export function dynamic_destructuring(unknown, aaa, bbb) {
  const { [unknown ? 'foo' : 'bar']: value } = {
    foo: aaa,
    bar: bbb
  };
  console.log(value);
}
