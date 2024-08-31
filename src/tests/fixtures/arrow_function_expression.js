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

export const fn4 = (a) => 4

const f5 = (a) => a ? (effect, 1) : 2
f5(true)
