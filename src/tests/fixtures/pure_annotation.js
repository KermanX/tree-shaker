/*#__PURE__*/ unknown1(1, other1());
/*#__PURE__*/ unknown2.g(1, other2(), /*#__PURE__*/ other3());
/*#__PURE__*/ unknown3?.g().h(1, other4());
export const a = unknown4.g?.(1, other5());

function simple() {
  effect();
  return a
}
/*#__PURE__*/ simple(1)
/*#__PURE__*/ simple(other1())
export const b = /*#__PURE__*/ simple(other2())
export const c = /*#__PURE__*/ simple(1, /*#__PURE__*/ other2())

function mutate(obj) {
  obj.a = b;
  delete obj.g;
  return obj.f();
}
/*#__PURE__*/ mutate({ a: 1, f: () => effect() })
export const d = /*#__PURE__*/ mutate({ f: effect })
