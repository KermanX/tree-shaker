# Tree Shaker Prototype

\[WIP\] This is a **prototype** of a tree shaker.

## Goal

Tree shake the following code:

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

## Approach

1. Parse the code.
2. Build the sematic information via `oxc_semantic`.
3. Tree shake the code.
  - Emulate the runtime behavior of the code. (Control flow, Side effects, ...)
  - Analyze the possible runtime values of the variables.
  - Remove the dead code.
4. Minify the code via `oxc_minifier`.
