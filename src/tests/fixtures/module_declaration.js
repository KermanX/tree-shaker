export function f1(a) {
  effect(a);
  return 1;
}

export const f2 = (a) => {
  effect(a);
  return 1;
}

export const t = 123;

export default t + "," + f2;
