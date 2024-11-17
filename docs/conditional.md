# 条件分支语句的分析

此处的条件分支语句，包括了：

- IfStatement: `if (a) { f() }`
- ConditionalExpression: `a ? f() : g()`
- LogicalExpression: `a && f()`
- AssignmentExpression with LogicalOperator: `a ||= f()`

分析后，我们需要确定条件分支语句的以下性质：

- 是否可能进入某个分支。比如 `if (false) { f() }` 是否可以删去
- 是否需要测试值。比如 `if (a) { f() }` 是否可以变为 `{ f() }`
- 对于 LogicalExpression，是否需要左侧的值。比如当 `f` 有副作用但始终返回 `0` 时，要把 `(f() + 1) && g()` 变为 `(f(), g())`。

需要注意的是，需要时刻牢记一段代码可能在分析的时候被多次调用（毕竟我们是模拟运行时的情况），但是在生成的代码中只能出现一次。

同时，一个分支有没有副作用，只有在整个分析完全结束之后才能确定。比如：

```js
let a = 1;
if (condition()) {
  a = 2; // 其实没有副作用
}
a = 3;
globalThis.x = a;
```

因此，我们也不可能在执行条件分支语句的当下就确定上述的任何性质。因此，唯一的方式是用 Consumable 来记录这些信息。

WIP
