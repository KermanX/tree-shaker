const fs = require('fs');
const path = require('path');

let input = '';

process.stdin.on('data', chunk => {
  input += chunk;
});

process.stdin.on('end', () => {
  const failedTests = 'FAILED TESTS\n' +
    input
    .replace(/^PASS.*$/gm, '')
    .replace(/^\[SKIP\].*$/gm, '')
    .replace(/^FAIL /gm, 'test' + path.sep)
    .replace(/ \(strict mode\)$/gm, '')
    .replace(/^.*\(default\)\n.*\n/gm, '')
    .replace(/\n{3,}/gm, '\n\n');
  fs.writeFileSync(path.join(__dirname, 'failed.txt'), failedTests);

  const stat = input.match(/^Ran \d+ tests[\s\S]+/m)[0];
  fs.writeFileSync(path.join(__dirname, 'stat.txt'), stat);
});
