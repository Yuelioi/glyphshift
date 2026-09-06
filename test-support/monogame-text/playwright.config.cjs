const path = require('node:path');
module.exports = {
  testDir: __dirname,
  testMatch: 'render.spec.cjs',
  timeout: 120000,
  workers: 1,
  retries: 0,
  reporter: 'list',
  outputDir: path.resolve(__dirname, '../../local-test/evidence/monogame-text/playwright'),
};
