export const fn1 = () => {
  let a = 1;
  return a;
}

export const fn2 = (a) => {
  let unused;
  return a;
}

export const fn3 = (a) => {
  let closure = (x) => x ? a: 2;
  return closure(false) + "a" + b;
}
