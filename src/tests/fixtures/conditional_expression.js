export function main(a, b) {
  t1 = 0 ? a : b;
  t1 = 1 ? a : b;

  t2 = 0 ? a : e;
  t2 = 1 ? a : e;

  t3 = 0 ? e : b;
  t3 = 1 ? e : b;

  t4 = (e, 0) ? a : b;

  0 ? e1 : b;
  1 ? e2 : b;

  0 ? a : e3;
  1 ? a : e4;

  (e5, 0) ? a : e6;
  (e7, 1) ? a : e8;

  a ? e9 : b;
  a ? b : e10;
}
