// @minify

export const result = {
  s1: typeof 1,
  s2: typeof 1n,
  s3: typeof 'a',
  s4: typeof true,
  s5: typeof null,
  s6: typeof undefined,
  s7: typeof {},
  // s8: typeof [],
  s9: typeof f1,
  // s10: typeof Symbol('a'),
  s11: typeof unknown,
  s12: typeof (unknown ? 'a' : 'b'),
  s13: typeof (unknown ? 'a' : 1),
}
