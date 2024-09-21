const fs = require('fs');
const path = require('path');

let input = '';

let ignored = JSON.parse(fs.readFileSync(path.join(__dirname, 'ignored.json'), 'utf8'));
let v8Failed = fs.readFileSync(path.join(__dirname, 'v8_test262.status'), 'utf8').split(/\r?\n/).filter(Boolean).map(s => s + '.js');

process.stdin.on('data', chunk => {
  input += chunk;
});

process.stdin.on('end', () => {
  const lines = input
    .replace(/^Ran \d+ tests[\s\S]+/m, '')
    .replace(/^PASS.*$/gm, '')
    .replace(/^\[SKIP\].*$/gm, '')
    .replace(/^FAIL /gm, '')
    .replace(/ \(strict mode\)$/gm, '')
    .replace(/^.*\(default\)\n.*\n/gm, '')
    .split('\n')
    .filter(Boolean);
  const failedTests = {}
  let expectedFailedNum = 0;
  for (let i = 0; i < lines.length; i+=2) {
    let name = lines[i].slice('test262/test/'.length);
    if (ignored.includes(name) || v8Failed.includes(name)) {
      expectedFailedNum++;
    } else {
      failedTests[name] = lines[i+1].replaceAll('\`', '\\\`') || '<NO OUTPUT>';
    }
  }

  const failedList = Object.entries(failedTests).map(([name, message]) => {
    return `[${name}](https://github.com/tc39/test262/tree/main/test/${name}): \`${message.trim()}\``;
  }).join('\n');
  fs.writeFileSync(path.join(__dirname, 'failed.txt'), failedList);

  const stat = input.match(/^Ran \d+ tests[\s\S]+/m)[0];
  const total = +stat.match(/^Ran (\d+) tests$/m)[1];
  const passedNum = +stat.match(/^(\d+) passed$/m)[1];
  const failedNum = Object.keys(failedTests).length;
  const restMessage = stat.match(/^Treeshake[\s\S]+/m)[0];
  fs.writeFileSync(path.join(__dirname, 'stat.txt'), `## Test262 Result

- Total: ${total}
- Passed: ${passedNum}
- Expected Failed: ${expectedFailedNum}
- New Failed: ${failedNum}
${restMessage.split('\n').filter(Boolean).map(s => `- ${s.trim()}`).join('\n')}

${failedNum ? `
<details>
<summary> New Failed Tests </summary>

${failedList}

</details>
` : ''}`);
});
