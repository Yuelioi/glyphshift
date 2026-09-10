'use strict';
const { test, expect } = require('../../apps/glyphshift-desktop/node_modules/@playwright/test');
const fs = require('node:fs');
const path = require('node:path');
const { createHash } = require('node:crypto');
const { connectNative } = require('./native-transport.cjs');

test('Runtime activation owns MV installation and removes its hooks on stop', async ({}, testInfo) => {
  test.skip(!process.env.GLYPHSHIFT_MV_MANAGED_BOOTSTRAP, 'Requires explicitly selected managed fixture');
  expect(process.env.GLYPHSHIFT_MV_REMOTE_HELPER).toBeTruthy();
  const root = process.env.GLYPHSHIFT_MV_RUNTIME_FIXTURE;
  const client = connectNative(process.env.GLYPHSHIFT_MV_NATIVE_DRIVER);
  const source = 'Today, we are going to explore something new.\nChoose a path when you are ready.';
  async function control(operation, generation = 1) {
    const file = operation === 'activate' ? `deployment-${generation}.json` : operation === 'update' ? `publication-${generation}.json` : null;
    const response = await client.runtime(operation, file ? fs.readFileSync(path.join(root, file), 'utf8') : '');
    expect(response.status, operation).toBe(0);
    return response;
  }
  async function bitmap() {
    return createHash('sha256').update(await client.evaluate('SceneManager._scene._messageWindow.contents._canvas.toDataURL()')).digest('hex');
  }
  async function enterDialogue() {
    await client.evaluate('window.gsMvPreviousScene=SceneManager._scene;DataManager.setupNewGame();SceneManager.goto(Scene_Map);true');
    await expect.poll(() => client.evaluate('!!(SceneManager._scene!==gsMvPreviousScene && SceneManager._scene instanceof Scene_Map && SceneManager._scene._messageWindow && SceneManager._scene._messageWindow._choiceWindow.active)'), { timeout: 15000 }).toBe(true);
    return bitmap();
  }
  async function screenshot(name) {
    fs.writeFileSync(testInfo.outputPath(name), Buffer.from(await client.capture(), 'base64'));
  }
  try {
    // The helper has only loaded the DLL. It has not installed engine hooks.
    await expect(client.evaluate('1')).rejects.toThrow('Native evaluation failed: 4');
    await control('session-start'); // read-only baseline instrumentation
    expect(await client.evaluate("!!window[Symbol.for('glyphshift.mv.fixture.engine.v1')]")).toBe(false);
    await client.evaluate('window.gsMvOriginalMessageStart=Window_Message.prototype.startMessage;true');
    const original = await enterDialogue();
    await screenshot('01-original.png');
    await client.evaluate("document.title='Other synthetic context';setTimeout(function(){document.title='Glyphshift MV Contract';},6500);true");
    await control('session-stop');
    const deployment = fs.readFileSync(path.join(root, 'deployment-1.json'), 'utf8');
    // Failed engine bootstrap must roll back Runtime capture and native hooks;
    // the same process can retry after its normal event loop restores the realm.
    expect((await client.runtime('activate', deployment)).status).toBe(14);
    await expect(client.evaluate('1')).rejects.toThrow('Native evaluation failed: 4');
    // No installMessageAdapter or process.dlopen expression is sent by the test.
    await control('activate');
    expect(await client.evaluate('Window_Message.prototype.startMessage===gsMvOriginalMessageStart')).toBe(false);
    const first = await enterDialogue();
    expect(first).not.toBe(original);
    expect(await client.evaluate('$gameMessage.allText()')).toBe(source);
    expect((await control('capture')).batch.records).toEqual([{ sequence: 1, adapterId: 'fixture.rpgmaker-mv.runtime-bridge', source }]);
    await screenshot('02-runtime-installed.png');
    await control('update', 2);
    const second = await enterDialogue();
    expect(second).not.toBe(first);
    expect((await control('capture')).batch.records).toEqual([{ sequence: 2, adapterId: 'fixture.rpgmaker-mv.runtime-bridge', source }]);
    await screenshot('03-updated.png');
    for (let cycle = 0; cycle < 4; cycle++) {
      await control('deactivate');
      await expect(client.evaluate('1')).rejects.toThrow('Native evaluation failed: 4');
      await control('session-start'); // inspect after activation has stopped itself
      expect(await client.evaluate('Window_Message.prototype.startMessage===gsMvOriginalMessageStart')).toBe(true);
      expect(await enterDialogue()).toBe(original);
      await control('session-stop');
      await control('activate', 2);
      expect(await enterDialogue()).toBe(second);
      expect((await control('capture')).batch.records).toEqual([{ sequence: 1, adapterId: 'fixture.rpgmaker-mv.runtime-bridge', source }]);
    }
    await control('deactivate');
    await control('session-start');
    expect(await enterDialogue()).toBe(original);
    await screenshot('04-runtime-restored.png');
    fs.writeFileSync(testInfo.outputPath('managed-activation.json'), JSON.stringify({
      bootstrapOwner: 'Native Adapter activation', driverInstalledMessageHook: false,
      originalFunctionRestored: true, cycles: 4, dictionaryGenerations: 2,
      failedBootstrapRolledBack: true, retryAfterTimeout: 'passed',
    }, null, 2));
  } finally {
    await client.runtime('deactivate').catch(() => {});
    await client.close();
  }
});
