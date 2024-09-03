// @ts-check

const { treeShake } = require('@kermanx/tree-shaker')

function treeShakeEval(input) {
  return input.replace(/eval\('(.*)'\)/, (_, content) => {treeShake(content, true, true)});
}

module.exports = function(test) {
  try {
    console.log(test.file);
    const index = test.contents.match(/\/\*---\r?\ndescription: >/m).index;
    if (index < 0) {
      throw new Error('Could not find the description comment');
    }
    test.contents = test.contents.slice(0, index) + treeShake(treeShakeEval(test.contents.slice(index)), true, false);
  } catch (error) {
    test.result = {
      stderr: `${error.name}: ${error.message}\n`,
      stdout: '',
      error
    };
  }

  return test;
};
