const glob = require('glob');
const swc = require('@swc/core');
const acorn = require('acorn');
const fs = require('fs');
const babel = require('@babel/core');
const files = glob.sync('./node_modules/three/src/**/*', { absolute: true, nodir: true });
const codes = files.map((x) => fs.readFileSync(x, 'utf-8'));
async function main() {
  console.time('swc_code');
  for (const code of codes) {
    swc.parseSync(code);
  }
  console.timeEnd('swc_code');
  console.time('swc_serial');
  for (const file of files) {
    await swc.parseFile(file);
  }
  console.timeEnd('swc_serial');
  console.time('swc_parallel');
  let queue = [];
  for (const file of files) {
    queue.push(swc.parseFile(file));
  }
  await Promise.all(queue);
  console.timeEnd('swc_parallel');
  console.time('acorn');
  for (const code of codes) {
    acorn.parse(code, { sourceType: 'module', ecmaVersion: 'latest' });
  }
  console.timeEnd('acorn');
  console.time('babel');
  for (const code of codes) {
    babel.parse(code);
  }
  console.timeEnd('babel');
}

main();
