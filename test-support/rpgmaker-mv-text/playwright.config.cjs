const path = require('node:path');
module.exports = {
  testDir: __dirname,
  testMatch: process.env.GLYPHSHIFT_MV_MANAGED_BOOTSTRAP ? 'managed-runtime.spec.cjs' : 'runtime.spec.cjs',
  timeout: 60000,
  workers: 1,
  retries: 0,
  reporter: 'list',
  outputDir: path.resolve(__dirname, '../../local-test/evidence/rpgmaker-mv/playwright'),
};
