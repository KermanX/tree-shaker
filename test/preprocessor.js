const { treeShake } = require('@kermanx/tree-shaker')

module.exports = function(test) {
  try {
    test.contents = treeShake(test.contents, true);
  } catch (error) {
    test.result = {
      stderr: `${error.name}: ${error.message}\n`,
      stdout: '',
      error
    };
  }

  return test;
};
