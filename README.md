# Experimental Tree Shaker

\[WIP\] This is an experimental tree shaker for JS based on [Oxc](https://oxc.rs).

[**Try online**](https://kermanx.github.io/tree-shaker/)

## Features

- Simulate the runtime behavior of the code, instead of applying rules.
- Single AST pass - Analyzer as much information as possible.
- As accurate as possible. [test262](https://github.com/tc39/test262) is used for testing.
- May not be the fastest. (But I will try my best)

## Examples

### Constant Folding

> This is a simple example, but it's a good start.

<table><tbody><tr><td width="500px"> Before </td><td> After </td></tr><tr>
<td valign="top">

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

</td><td valign="top">

```js
export function f() {
  return 1;
}
```

</td></tr></tbody></table>

### Dead Code Elimination

<table><tbody><tr><td width="500px"> Before </td><td> After </td></tr><tr>
<td valign="top">

```js
function f(value) {
  if (value) console.log(`${value} is truthy`);
}
f(1);
f(0);

function g(t1, t2) {
  if (t1 && t2) console.log(2);
  else if (t1 || t2) console.log(1);
  else console.log(0);
}
g(true, true);
g(false, false);
```

</td><td valign="top">

```js
function f() {
  {
    console.log("1 is truthy");
  }
}
f();

function g(t1) {
  if (t1 && true) console.log(2);
  else {
    console.log(0);
  }
}
g(true);
g(false);
```

</td></tr></tbody></table>

### Object Property Mangling

> This is beyond the scope of tree-shaking, we need a new name for this project 😇.

<table><tbody><tr><td width="500px"> Before </td><td> After </td></tr><tr>
<td valign="top">

```js
export function main() {
  const obj = {
    foo: v1,
    [t1 ? "bar" : "baz"]: v2,
  };
  const key = t2 ? "foo" : "bar";
  console.log(obj[key]);
}
```

</td><td valign="top">

```js
export function main() {
  const obj = {
    a: v1,
    [t1 ? "b" : "c"]: v2,
  };
  const key = t2 ? "a" : "b";
  console.log(obj[key]);
}
```

</td></tr></tbody></table>

### JSX

> `createElement` also works, if it is directly imported from `react`.

<table><tbody><tr><td width="500px"> Before </td><td> After </td></tr><tr>
<td valign="top">

```jsx
function Name({ name, info }) {
  return (
    <span>
      {name}
      {info && <sub> Lots of things never rendered </sub>}
    </span>
  );
}
export function Main() {
  return <Name name={"world"} />;
}
```

</td><td valign="top">

```jsx
function Name() {
  return (
    <span>
      {"world"}
      {}
    </span>
  );
}
export function Main() {
  return <Name />;
}
```

</td></tr></tbody></table>

### React.js

<table><tbody><tr><td width="500px"> Before </td><td> After </td></tr><tr>
<td valign="top">

```jsx
import React from "react";
const MyContext = React.createContext("default");
function Inner() {
  const value = React.useContext(MyContext);
  return <div>{value}</div>;
}
export function main() {
  return (
    <MyContext.Provider value="hello">
      <Inner />
    </MyContext.Provider>
  );
}
```

</td><td valign="top">

```jsx
import React from "react";
const MyContext = React.createContext();
function Inner() {
  return <div>{"hello"}</div>;
}
export function main() {
  return (
    <MyContext.Provider>
      <Inner />
    </MyContext.Provider>
  );
}
```

</td></tr></tbody></table>

## Comparison

- **Rollup**: Rollup tree-shakes the code in a multi-module context, while this project is focused on a single module. For some cases, this project can remove 10% more code than Rollup.
- **Closure Compiler**: Closure Compiler can be considered as a tree shaker + minifier, while this project is only a tree shaker (for the minifier, we have `oxc_minifier`). Theoretically, we can shake more than Closure Compiler, but we cannot compare them directly because we don't have a equivalent minifier. Also, it's written in Java, which is hard to be integrated into the JS ecosystem.
- **swc**: swc can also be considered as a tree shaker + minifier. TBH, currently swc is much faster and more complete. It is rule-based, which is a different approach from this project. It's also not compatible with the Oxc project, thus a new tree shaker is needed.

## Todo

- Performance!
- JS Builtins metadata
- Test against fixtures from other tree shakers like Rollup
- Rollup-like try-scope optimization/de-optimization
- Reuse code with oxc_minifier for JS computation logics
- Type narrowing
- Pure annotation

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
