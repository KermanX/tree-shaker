const { treeShake } = require('@kermanx/tree-shaker')

function treeShakeEval(input) {
  return input.replace(/eval\('(.*)'\)/, (_, content) => {treeShake(content, true, true)});
}

module.exports = function(test) {
  try {
    test.contents = treeShake(treeShakeEval(test.contents), true, false);
  } catch (error) {
    test.result = {
      stderr: `${error.name}: ${error.message}\n`,
      stdout: '',
      error
    };
  }

  return test;
};
