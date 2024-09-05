class A extends (1+1) {
  constructor(a, b) {
    super(a);
    b();
  }
  fn(a) {
    a;
    a = 1;
    a();
  }
  [1+1] = 1+1
  get [3+3]() { return 3+3 }
  set x(v) { this._x = v }
}

const a = class X { [1+1] = 1+1 };
const b = class extends (1+1) { [1+1] = 1+1 };

export default class{
  a = 1;
}