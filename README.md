# Tree Shaker Prototype

\[WIP\] This is a **prototype** of a JavaScript tree shaker.

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

May not be very fast, but should generates the least code possible.

## Approach

1. Parse the code via `oxc_parser`.
2. Build the sematic information via `oxc_semantic`.
3. Tree shake the code.
    - Emulate the runtime behavior of the code. (Control flow, Side effects, ...)
    - Analyze the possible runtime values of the variables.
    - Remove the dead code.
4. Minify the code via `oxc_minifier`.

> Tree shake v.s. Minify:
>
> In this project, tree shake is to remove the dead code, while minify is to reduce the code size. They are different steps in the process.
