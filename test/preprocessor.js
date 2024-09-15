// @ts-check

const { treeShake } = require('@kermanx/tree-shaker')
const pc = require("picocolors");
const Diff = require('diff')

const do_minify = false;

function treeShakeEval(input, tree_shake) {
  return input.replace(/eval\('(.*)'\)/, (_, content) => {treeShake(content, tree_shake, do_minify, true)});
}

function printDiff(diff) {
  let t1 = ""
  diff.forEach((part) => {
    // green for additions, red for deletions
    t1 += part.added ? "" :
               part.removed ? pc.bgRed(part.value) :
                              part.value;
  });
  console.log("OLD", t1);
  
  let t2 = ""
  diff.forEach((part) => {
    // green for additions, red for deletions
    t2 += part.added ? pc.bgGreen(part.value) :
               part.removed ? "" :
                              part.value;
  });
  console.log("NEW", t2);
}

let index = 0;
module.exports = function(test) {
  try {
    console.log('\n>', ++index, '----------------');
    console.log(test.file);
    let prelude = test.contents.slice(0, test.insertionIndex);
    let main = test.contents.slice(test.insertionIndex);
    let minified = treeShake(treeShakeEval(main, false), false, do_minify, false);
    let startTime = Date.now();
    let treeShaked = treeShake(treeShakeEval(main, true), true, do_minify, false);
    let endTime = Date.now();
    console.log(`${pc.gray(main.length)} -> ${pc.red(minified.length)} -> ${pc.green(treeShaked.length)} (${pc.yellow((treeShaked.length * 100 / minified.length).toFixed(2) + '%')}) +${endTime - startTime}ms`);
    if (minified !== treeShaked)
      printDiff(Diff.diffChars(minified, treeShaked));
    test.contents = prelude + treeShaked;
  } catch (error) {
    test.result = {
      stderr: `${error.name}: ${error.message}\n`,
      stdout: '',
      error
    };
  }

  return test;
};
