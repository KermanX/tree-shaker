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
let skipped = 0;
module.exports = function(test) {
  try {
    let prelude = test.contents.slice(0, test.insertionIndex);
    let main = test.contents.slice(test.insertionIndex);

    if (main.includes('eval(')) {
      console.log('\n> Skipping eval', ++skipped, '----------------');
      console.log(test.file);
      return test;
    }
    if (main.includes('$DONOTEVALUATE')) {
      console.log('\n> Skipping $DONOTEVALUATE', ++skipped, '----------------');
      console.log(test.file);
      return test;
    }
    if (/with\s*\(/.test(main)) {
      console.log('\n> Skipping with', ++skipped, '----------------');
      console.log(test.file);
      return test;
    }

    console.log('\n> Testing', ++index, '----------------');
    console.log(test.file);
    let minified = treeShake(treeShakeEval(main, false), false, do_minify, false);
    let startTime = Date.now();
    let treeShaked = treeShake(treeShakeEval(main, true), true, do_minify, false);
    let endTime = Date.now();
    console.log(`${pc.gray(main.length)} -> ${pc.red(minified.length)} -> ${pc.green(treeShaked.length)} (${pc.yellow((treeShaked.length * 100 / minified.length).toFixed(2) + '%')}) +${endTime - startTime}ms`);
    // if (minified !== treeShaked && !test.file.includes('unicode'))
    //   printDiff(Diff.diffChars(minified.slice(0, 500), treeShaked.slice(0, 500)));
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
