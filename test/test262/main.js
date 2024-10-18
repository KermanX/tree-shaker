// @ts-check

import { treeShake } from '@kermanx/tree-shaker'
import { readFileSync, writeFileSync } from 'fs'
import * as prettier from 'prettier'

const input = readFileSync('./input.js', 'utf8');

const start = Date.now();
const result = treeShake(input, "recommended", false, false);
console.log('Time:', Date.now() - start + 'ms');

writeFileSync('./output.js', result.output);
writeFileSync('./diagnostics.txt', result.diagnostics.join('\n'));
