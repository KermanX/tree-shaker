const fs = require('fs');
const path = require('path');

let input = '';

process.stdin.on('data', chunk => {
  input += chunk;
});

process.stdin.on('end', () => {
  const output = input.replace(/^PASS.*$/gm, '')
    .replace(/^\[SKIP\].*$/gm, '')
    .replace(/^FAIL /gm, 'test\\')
    .replace(/ \(strict mode\)$/gm, '')
    .replace(/^.*(default)\n.*\n/gm, '')
    .replace(/\n{3,}/gm, '\n\n');
  const resultPath = path.join(__dirname, 'result.txt');
  try {
    fs.writeFileSync(resultPath, output);
    console.log(`Processed output written to ${resultPath}`);
  } catch (err) {
    console.error(`Error writing to file: ${err.message}`);
  }
});
