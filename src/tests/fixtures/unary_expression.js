// @minify

export function test_typeof(unknown) {
  return {
    s1: typeof 1,
    s2: typeof 'a',
    s3: typeof true,
    // s4: typeof null,
    s5: typeof undefined,
    s6: typeof {},
    // s7: typeof [],
    s8: typeof f1,
    // s9: typeof Symbol('a'),
    s10: typeof unknown,
    s11: typeof (unknown ? 'a' : 'b'),
    s12: typeof (unknown ? 'a' : 1),
  }
}