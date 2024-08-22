fn tree_shake(input: &str) {
  let result = crate::tree_shake(input);
  let output = result.codegen_return.source_text;
  insta::assert_snapshot!(output);
}

#[test]
fn test_1() {
  tree_shake(
    r#"
      export let a = 1 && 2;
      export { b };
      let c = 3;
      let { ["b"]: b, d } = { b: 2, d: effect };
      "#,
  );
}
