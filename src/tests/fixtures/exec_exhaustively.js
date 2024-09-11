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

export function f3() {
  label: for (var i = 0; i < 10; ++i) {
    let x = 'middle' + i;
    for (var j = 0; j < 10; ++j) {
      let x = 'inner' + j;
      continue label;
    }
  }
}
