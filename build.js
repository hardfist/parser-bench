const glob = require('glob');
const swc = require('@swc/core');
const acorn = require('acorn');
const esbuild = require('esbuild');
const fs = require('fs');
const babel = require('@babel/core');
const files = glob.sync('./node_modules/three/src/**/*', { absolute: true, nodir: true });
const list = files.map((x) => ({ file: x, code: fs.readFileSync(x, 'utf-8') }));
let codes = [];
for (let i = 0; i < 10; i++) {
  codes = codes.concat(list);
}
async function main() {
  console.time('swc_code');
  for (const { _, code } of codes) {
    swc.parseSync(code);
  }
  console.timeEnd('swc_code');
  console.time('swc_serial');
  for (const { file } of codes) {
    await swc.parseFile(file);
  }
  console.timeEnd('swc_serial');
  console.time('swc_parallel');
  let queue = [];
  for (const { file } of codes) {
    queue.push(swc.parseFile(file));
  }
  await Promise.all(queue);
  console.timeEnd('swc_parallel');
  console.time('acorn');
  for (const { code } of codes) {
    acorn.parse(code, { sourceType: 'module', ecmaVersion: 'latest' });
  }
  console.timeEnd('acorn');
  console.time('babel');
  for (const { code } of codes) {
    babel.parse(code);
  }
  console.timeEnd('babel');

  console.time('esbuild');
  for (const { code } of codes) {
    await esbuild.transform(code);
  }
  console.timeEnd('esbuild');
}

main();
