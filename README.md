# Tree Shaker

\[WIP\] This is an experimental tree shaker for JS based on [Oxc](https://oxc.rs).

> If this project goes well, I personally hope it can become part of the Oxc or the Rolldown project.

- **Try it online**: https://kermanx.github.io/tree-shaker/
- **Test262 Result**: Goto [commits](https://github.com/KermanX/tree-shaker/commits/main/) and view the latest comment

## Goal

Tree shake the following code (this already works!):

```js
export function f() {
  function g(a) {
    if (a)
      console.log('effect')
    else
      return 'str'
  }
  let { ["x"]: y = 1 } = { x: g('') ? undefined : g(1) }
  return y
}
```

To:

```js
export function f() {
  return 1
}
```

## Todo

- Performance!
- Test against fixtures from other tree shakers like Rollup
- Implement built-in objects and properties
- Rollup-like try-scope optimization/de-optimization
- Reuse code with oxc_minifier for JS computation logics
- Type narrowing
- Multiple-module support

## Approach

1. Parse the code via `oxc_parser`.
2. Build the semantic information via `oxc_semantic`.
3. Tree shake the code.
    - Emulate the runtime behavior of the code. (Control flow, Side effects, ...)
    - Analyze the possible runtime values of the variables.
    - Remove the dead code.
4. Minify the code via `oxc_minifier`. (Optional)

### Concepts

- `Entity`: Represents the analyzed information of a JS value.
- `Consumable`: Entity or AST Nodes or some other things that the runtime value of `Entity` depends on.
- Scopes:
    - Call Scope: Function call scope.
    - Cf Scope: Control flow scope.
    - Variable Scope: Variable scope.
    - Try Scope: Try statement or function.
