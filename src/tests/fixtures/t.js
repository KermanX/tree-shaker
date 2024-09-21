var iter = {};
iter[Symbol.iterator] = function() {
  var nextCount = 3;
  return {
    next: function() {
      nextCount += 1;
      return { done: nextCount === 6, value: nextCount };
    }
  };
};

var callCount = 0;

(function(a,b,c) {
  assert.sameValue(arguments.length, 5);
  assert.sameValue(arguments[0], 1);
  assert.sameValue(arguments[1], 2);
  assert.sameValue(arguments[2], 3);
  assert.sameValue(arguments[3], 4);
  assert.sameValue(arguments[4], 5);
  callCount += 1;
}.apply(null, [1, 2, 3, ...iter]));

assert.sameValue(callCount, 1);