# 执行时依赖

除了进行运算有依赖（比如 `a + b` 的结果依赖于 `a` 和 `b`），运行一段代码也有依赖。

比如：

```js
if (a)
  effect()
```

其中，`effect` 的执行依赖于 `a` 的值为真值。由于还要进行其他优化，故对于 if 语句没有这么简单，下面是一个看起来更复杂的例子：

```js
const obj = { x: 1 }
({ [effect()]: _unused } = obj)
```

其中，`effect` 的执行依赖于 `obj` 是一个可以解构的 object，否则运行时直接报错了。我们只能将其优化为

```js
const obj = {}
({ [effect()]: _unused } = obj)
```

> 为什么不优化到只有 `effect()`？因为这属于 Minifier 的工作。与 Tree Shaker 的实现逻辑无关。

WIP
