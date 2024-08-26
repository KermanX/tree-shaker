fn tree_shake(input: &str) -> String {
  let result = crate::tree_shake(input);
  result.codegen_return.source_text
}

macro_rules! tree_shake_snapshot {
  ($input: expr $(,)?) => {
    let result = tree_shake($input);
    insta::assert_snapshot!(result);
  };
}

#[test]
fn test_1() {
  tree_shake_snapshot!(
    r#"
export let a = 1 && 2;
export { b };
let c = 3;
let { ["b"]: b, d } = { b: 2, d: effect };

let t = 0;
if (t) {
  effect3;
}
else {
  effect4;
}

if (effect) {
  effect5;
}

while(0) {
  effect6;
}

while(effect7) {
  0;
}"#,
  );
}

#[test]
fn test_2() {
  tree_shake_snapshot!(
    r#"
let x = 1;

function f1(a) {
  function closure() {
    return x;
  }

  if (a)
    return closure();
  else
    effect;
}

export const t = f1(true);

export function f2() {
  effect;
}

f2();

const r = f2();
"#,
  );
}
