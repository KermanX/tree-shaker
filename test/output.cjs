const fs = require('fs');
const path = require('path');

let input = '';

let ignored = JSON.parse(fs.readFileSync(path.join(__dirname, 'ignored.json'), 'utf8'));

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
    .replace(/\n{2,}/gm, '\n')
    .split('\n');
  const failedTests = {}
  for (let i = 0; i < lines.length; i+=2) {
    let name = lines[i].slice('test262/test/'.length);
    if (!ignored.includes(name)) {
      failedTests[name] = lines[i+1];
    }
  }

  const failedList = Object.entries(failedTests).map(([name, message]) => {
    const basename = path.basename(name);
    return `[${basename}](https://github.com/tc39/test262/tree/main/test/${name}): ${message}\n`;
  }).join('\n');
  fs.writeFileSync(path.join(__dirname, 'failed.txt'), failedList);

  const stat = input.match(/^Ran \d+ tests[\s\S]+/m)[0];
  const total = +stat.match(/^Ran (\d+) tests$/m)[1];
  const passedNum = +stat.match(/^(\d+) passed$/m)[1];
  const ignoredNum = ignored.length;
  const failedNum = Object.keys(failedTests).length;
  const restMessage = stat.match(/^Treeshake[\s\S]+/m)[0];
  fs.writeFileSync(path.join(__dirname, 'stat.txt'), `## Test262 Result

- Total: ${total}
- Passed: ${passedNum}
- Expected Failed: ${ignoredNum}
- New Failed: ${failedNum}
${restMessage.split('\n').filter(Boolean).map(s => `- ${s.trim()}`).join('\n')}

<details>
<summary> New Failed Tests </summary>

${failedList}

</details>
`);
});
