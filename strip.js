const strip = require('strip-comments');
const fs = require('fs');
const path = require('path');
const code = fs.readFileSync('/Users/yangjian/github/parser-bench/node_modules/bizcharts/umd/BizCharts.js', 'utf-8');
const striped_code = strip(code);
fs.writeFileSync(path.resolve(__dirname, './tmp.js'), striped_code);
console.log('diff:', code.length - striped_code.length);
