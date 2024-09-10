export function f1() {
  let x = false;
  let y = 0;
  while(someCondition()) {
    if (x) {
      y++;
    }
    x = true;
    effect(y);
  }
}

export function f2(q) {
  let a = true
  while (sth()) {
    a = true
    if (a) effect1(); else effect2();
  }
  if (a) effect1(); else effect2();
}