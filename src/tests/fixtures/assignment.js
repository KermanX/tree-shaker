export function f1() {
  let a = 1;
  a = 2;
  return a;
}

export function f2(a) {
  a = 2;
  return a;
}

export function f3(a, b) {
  if (b) a = 2;
  return a;
}
