# Experimental Tree Shaker

\[WIP\] This is an experimental tree shaker for JS based on [Oxc](https://oxc.rs).

> If this project goes well, I personally hope it can become part of the Oxc or the Rolldown project.

- **Try it online**: https://kermanx.github.io/tree-shaker/
- **Test262 Result**: Goto [commits](https://github.com/KermanX/tree-shaker/commits/main/) and view the latest comment

## Features

- Simulate the runtime behavior of the code, instead of applying rules.
- Single AST pass - Analyzer as much information as possible.
- As accurate as possible. [test262](https://github.com/tc39/test262) is used for testing.
- May not be the fastest. (But I will try my best)
- Will have the ability to **tree-shake React components props** soon

## Demo

Tree shake the following code (this already works!):

```js
export function f() {
  function g(a) {
    if (a) console.log("effect");
    else return "str";
  }
  let { ["x"]: y = 1 } = { x: g("") ? undefined : g(1) };
  return y;
}
```

To:

```js
export function f() {
  return 1;
}
```

Although the code above is very simple to analyze, but achieving this effect correctly for every codebase requires a lot of work - JS is too dynamic.

## Comparison

- **Rollup**: Rollup is great but somehow I couldn't understand its codebase. I hope this project can be more readable and maintainable. Currently, for the same code, this project can produce code roughly 2% smaller than Rollup (both after minification via `uglify-js`) for a Vue starter app.
- **Closure Compiler**: Closure Compiler can be considered as a tree shaker + minifier, while this project is only a tree shaker (for the minifier, we have `oxc_minifier`). Theoretically, we can shake more than Closure Compiler, but we cannot compare them directly because we don't have a equivalent minifier. Also, it's written in Java, which is hard to be integrated into the JS ecosystem.
- **swc**: swc can also be considered as a tree shaker + minifier. TBH, currently swc is much faster than this project and more complete. But it's rule-based, which is a different approach from this project. It's also not compatible with the Oxc project, thus a new tree shaker is needed.

## Todo

- Performance!
- JS Builtins metadata
- Test against fixtures from other tree shakers like Rollup
- Rollup-like try-scope optimization/de-optimization
- Reuse code with oxc_minifier for JS computation logics
- Type narrowing
- Multiple-module support

## Basic Approach

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
