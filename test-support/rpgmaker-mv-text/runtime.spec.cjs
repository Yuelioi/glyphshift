'use strict';
const { test, expect } = require('../../apps/glyphshift-desktop/node_modules/@playwright/test');
const fs = require('node:fs');
const path = require('node:path');
const { createHash } = require('node:crypto');

// Chrome 65 predates Playwright's Browser CDP requirements. Use its page protocol
// inside the repository test runner, with an explicitly authorized local fixture.
async function connect(endpoint) {
  const url = new URL(endpoint);
  if (url.hostname !== '127.0.0.1' || url.protocol !== 'http:') throw new Error('Local endpoint required');
  const response = await fetch(new URL('/json', url));
  const pages = (await response.json()).filter(page => page.type === 'app' && page.title === 'Glyphshift MV Contract');
  expect(pages).toHaveLength(1);
  const socket = new WebSocket(pages[0].webSocketDebuggerUrl);
  await new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error('CDP open timeout')), 5000);
    socket.addEventListener('open', () => { clearTimeout(timer); resolve(); }, { once: true });
    socket.addEventListener('error', () => { clearTimeout(timer); reject(new Error('CDP open failed')); }, { once: true });
  });
  let sequence = 0;
  const pending = new Map();
  socket.addEventListener('message', event => {
    const message = JSON.parse(event.data);
    const request = pending.get(message.id);
    if (!request) return;
    pending.delete(message.id);
    clearTimeout(request.timer);
    if (message.error) request.reject(new Error(JSON.stringify(message.error)));
    else request.resolve(message.result);
  });
  function send(method, params = {}) {
    return new Promise((resolve, reject) => {
      const id = ++sequence;
      const timer = setTimeout(() => { pending.delete(id); reject(new Error(`CDP timeout: ${method}`)); }, 5000);
      pending.set(id, { resolve, reject, timer });
      socket.send(JSON.stringify({ id, method, params }));
    });
  }
  return {
    send,
    async capture() { return (await send('Page.captureScreenshot', { format: 'png' })).data; },
    async evaluate(expression) {
      const result = await send('Runtime.evaluate', { expression, returnByValue: true });
      if (result.exceptionDetails) throw new Error(JSON.stringify(result.exceptionDetails));
      return result.result.value;
    },
    close() {
      for (const request of pending.values()) { clearTimeout(request.timer); request.reject(new Error('CDP closed')); }
      pending.clear();
      socket.close();
    },
  };
}

test('authorized MV runtime: full dialogue, two translations, next-message restore', async ({}, testInfo) => {
  const endpoint = process.env.GLYPHSHIFT_MV_CDP;
  const driver = process.env.GLYPHSHIFT_MV_NATIVE_DRIVER;
  const runtimeRoot = process.env.GLYPHSHIFT_MV_RUNTIME_FIXTURE;
  test.skip(!endpoint && !driver, 'Requires an authorized local MV fixture and an explicit transport');
  if (endpoint && driver) throw new Error('Select exactly one transport');
  if (runtimeRoot && !driver) throw new Error('Runtime fixture requires native transport');
  const client = driver ? require('./native-transport.cjs').connectNative(driver) : await connect(endpoint);
  async function runtime(operation, generation = 1) {
    const json = operation === 'activate' || operation === 'update'
      ? fs.readFileSync(path.join(runtimeRoot, `${operation === 'activate' ? 'deployment' : 'publication'}-${generation}.json`), 'utf8') : '';
    const result = await client.runtime(operation, json);
    expect(result.status, `Runtime ${operation}`).toBe(0);
    return result;
  }
  try {
    const identity = await client.evaluate('({engine:Utils.RPGMAKER_NAME,version:Utils.RPGMAKER_VERSION,nw:process.versions.nw,chromium:process.versions.chromium,title:$dataSystem.gameTitle})');
    expect(identity.engine).toBe('MV');
    expect(identity.title).toBe('Glyphshift MV Contract');
    if (driver) {
      // V8 exceptions must be caught locally without poisoning the next callback.
      await expect(client.evaluate('throw new Error("fixture error")')).rejects.toThrow('fixture error');
      await expect(client.evaluate('var = invalid syntax')).rejects.toThrow('SyntaxError');
      for (let index = 0; index < 16; index++) {
        expect(await client.evaluate(`(${index}+1)`)).toBe(index + 1);
      }
    }
    fs.writeFileSync(testInfo.outputPath('runtime-identity.json'), JSON.stringify(identity, null, 2));
    const source = 'Today, we are going to explore something new.\nChoose a path when you are ready.';
    async function enterDialogue() {
      // The fixture has an authored autorun event. Starting a new game is a test
      // action, never the adapter's refresh implementation.
      await client.evaluate('window.gsMvPreviousScene=SceneManager._scene; DataManager.setupNewGame(); SceneManager.goto(Scene_Map); true');
      await expect.poll(() => client.evaluate('!!(SceneManager._scene !== gsMvPreviousScene && SceneManager._scene instanceof Scene_Map && SceneManager._scene._messageWindow && SceneManager._scene._messageWindow._choiceWindow.active)'), { timeout: 15000 }).toBe(true);
      return bitmapHash();
    }
    async function bitmapHash() {
      const bitmap = await client.evaluate('SceneManager._scene._messageWindow.contents._canvas.toDataURL()');
      return createHash('sha256').update(bitmap).digest('hex');
    }
    async function screenshot(name) {
      fs.writeFileSync(testInfo.outputPath(name), Buffer.from(await client.capture(), 'base64'));
    }
    const baseline = await enterDialogue();
    await screenshot('01-original.png');
    const adapter = fs.readFileSync(path.join(__dirname, 'adapter.cjs'), 'utf8');
    if (runtimeRoot) {
      expect(await client.evaluate('process.versions.node')).toBe('9.7.1');
      const bridge = path.resolve(runtimeRoot, 'glyphshift_mv_fixture.dll');
      if (!(await client.evaluate('!!(window.gsMvNativeModule && gsMvNativeModule.exports.resolve)'))) {
        const deployment = fs.readFileSync(path.join(runtimeRoot, 'deployment-1.json'), 'utf8');
        // Runtime loads and validates the DLL first. An unattached Node module
        // must fail activation cleanly without registering on the wrong thread.
        expect((await client.runtime('activate', deployment)).status).toBe(14);
        // Failed activation rolls back and unloads; retain a fixture reference
        // to separately prove Node can register an already-loaded DLL.
        expect((await client.runtime('preload')).status).toBe(0);
      }
      await client.evaluate(`window.gsMvLoadState='pending';setTimeout(function(){try{
        if (!(window.gsMvNativeModule && gsMvNativeModule.exports.resolve)) {
          window.gsMvNativeModule={exports:{}};process.dlopen(gsMvNativeModule,${JSON.stringify(bridge)});
        }
        gsMvLoadState='ready';
      }catch(error){gsMvLoadState=String(error);}},0);true`);
      await expect.poll(() => client.evaluate('gsMvLoadState')).toBe('ready');
      expect(await client.evaluate('typeof gsMvNativeModule.exports.resolve')).toBe('function');
      // The same already-loaded DLL is now ready for the Runtime host binding.
      await runtime('activate');
    }
    await client.evaluate(`window.gsMvExports = (function(){var module={exports:{}};${adapter}\nreturn module.exports;})();
      window.gsMvObserved=[];window.gsMvResolved=[];window.gsMvDictionary=Object.create(null);
      window.gsMvSession=gsMvExports.installMessageAdapter(Window_Message,{
        resolve:function(event){gsMvObserved.push(event);var value=${runtimeRoot
          ? "gsMvNativeModule.exports.resolve(event.source,event.usage==='draw'?1:3)"
          : "event.usage==='draw'?gsMvDictionary[event.source]:undefined"};gsMvResolved.push(value);return value;}
      });true`);
    await client.evaluate(`gsMvDictionary[${JSON.stringify(source)}]='今天，我们来探索新事物。\\n准备好后，选择一条路。';true`);
    const translated = await enterDialogue();
    expect(translated).not.toBe(baseline);
    expect(await client.evaluate('$gameMessage.allText()')).toBe(source);
    expect(await client.evaluate('gsMvResolved[0]')).toBe('今天，我们来探索新事物。\n准备好后，选择一条路。');
    expect(await client.evaluate('gsMvObserved')).toEqual([{ source, origin: 'display-entry', usage: 'draw' }]);
    await screenshot('02-translated.png');
    if (runtimeRoot) {
      const first = await runtime('capture');
      fs.writeFileSync(testInfo.outputPath('runtime-capture-first.json'), JSON.stringify(first.batch, null, 2));
      expect(first.batch).toMatchObject({ producerId: 'mv-runtime-contract', generation: 1, droppedTotal: 0,
        records: [{ sequence: 1, adapterId: 'fixture.rpgmaker-mv.runtime-bridge', source }] });
      expect(first.batch.records).toHaveLength(1);
      expect((await runtime('capture')).batch.records).toHaveLength(0);
      // Rejected bridge inputs do not pollute capture; observe-only evidence
      // captures once but must not return a translation even on a dictionary hit.
      expect(await client.evaluate(`[
        gsMvNativeModule.exports.resolve('',1),
        gsMvNativeModule.exports.resolve(new Array(16386).join('x'),1),
        gsMvNativeModule.exports.resolve('invalid-kind',2)
      ].every(function(value){return value===undefined;})`)).toBe(true);
      expect((await runtime('capture')).batch.records).toHaveLength(0);
      expect(await client.evaluate(`gsMvNativeModule.exports.resolve(${JSON.stringify(source)},3)===undefined`)).toBe(true);
      const observedOnly = (await runtime('capture')).batch.records;
      expect(observedOnly).toEqual([{ sequence: 2, adapterId: 'fixture.rpgmaker-mv.runtime-bridge', source }]);
      await runtime('update', 2);
    }
    await client.evaluate(`gsMvDictionary[${JSON.stringify(source)}]='今天，一起开始新的探索。\\n准备好了就选条路吧。';true`);
    const revised = await enterDialogue();
    expect(revised).not.toBe(translated);
    expect(await client.evaluate('gsMvResolved[1]')).toBe('今天，一起开始新的探索。\n准备好了就选条路吧。');
    await screenshot('03-next-generation.png');
    if (runtimeRoot) {
      const second = await runtime('capture');
      fs.writeFileSync(testInfo.outputPath('runtime-capture-second.json'), JSON.stringify(second.batch, null, 2));
      expect(second.batch.records).toEqual([{ sequence: 3, adapterId: 'fixture.rpgmaker-mv.runtime-bridge', source }]);
      await runtime('deactivate');
      expect(await client.evaluate(`gsMvNativeModule.exports.resolve(${JSON.stringify(source)},1)===undefined`)).toBe(true);
    }
    // With the real Runtime, leave the JS hook installed to prove the host's
    // deactivation itself restores the next message.
    if (!runtimeRoot) await client.evaluate('gsMvSession.stop();true');
    expect(await bitmapHash()).toBe(revised);
    const restored = await enterDialogue();
    expect(restored).toBe(baseline);
    expect(await client.evaluate('gsMvObserved.length')).toBe(runtimeRoot ? 3 : 2);
    await screenshot('04-next-message-restored.png');
    if (runtimeRoot) {
      const firstPublication = fs.readFileSync(path.join(runtimeRoot, 'publication-1.json'), 'utf8');
      const secondPublication = fs.readFileSync(path.join(runtimeRoot, 'publication-2.json'), 'utf8');
      const expectedTranslation = '今天，一起开始新的探索。\n准备好了就选条路吧。';
      // Repeated control operations must neither leave stale host bindings nor
      // accumulate native capture producers. No event replay is needed here.
      for (let cycle = 0; cycle < 12; cycle++) {
        await runtime('activate', 2);
        await runtime('activate', 2);
        expect((await client.runtime('update', firstPublication)).status).toBe(16);
        expect((await client.runtime('update', '{invalid-json')).status).toBe(2);
        const conflict = JSON.parse(secondPublication);
        conflict.translations[0].translation = 'Conflicting same-generation text';
        expect((await client.runtime('update', JSON.stringify(conflict))).status).toBe(16);
        // Publication updates require a strictly newer generation, unlike
        // idempotent activation of the same deployment.
        expect((await client.runtime('update', secondPublication)).status).toBe(16);
        expect(await client.evaluate(`gsMvNativeModule.exports.resolve(${JSON.stringify(source)},1)`)).toBe(expectedTranslation);
        const batch = (await runtime('capture')).batch;
        expect(batch).toMatchObject({ generation: 2, droppedTotal: 0 });
        expect(batch.records).toEqual([{ sequence: 1, adapterId: 'fixture.rpgmaker-mv.runtime-bridge', source }]);
        await runtime('deactivate');
        // The existing control protocol reports RuntimeUnavailable when it is
        // already stopped. That rejection must not prevent later activation.
        expect((await client.runtime('deactivate')).status).toBe(15);
        expect(await client.evaluate(`gsMvNativeModule.exports.resolve(${JSON.stringify(source)},1)===undefined`)).toBe(true);
      }
      await runtime('activate', 2);
      expect(await enterDialogue()).toBe(revised);
      await screenshot('05-reactivated.png');
      await runtime('deactivate');
      expect(await enterDialogue()).toBe(baseline);
      fs.writeFileSync(testInfo.outputPath('runtime-lifecycle.json'), JSON.stringify({
        cycles: 12, duplicateActivation: 'passed', duplicateStop: 'inactive-status-15',
        oldPublication: 'rejected', conflictingPublication: 'rejected',
        invalidPublication: 'rejected', reactivatedPixels: 'passed', restoredPixels: 'passed',
      }, null, 2));
      if (process.env.GLYPHSHIFT_MV_NATIVE_SESSION_DLL) {
        if (process.env.GLYPHSHIFT_MV_REMOTE_HELPER) {
          const { spawnSync } = require('node:child_process');
          const executable = process.env.GLYPHSHIFT_MV_EXE;
          for (const [targetPath, reason] of [
            [path.join(path.dirname(executable), 'synthetic-absent.exe'), 'Executable identity changed'],
            [executable, 'Target instance changed'],
          ]) {
            const rejected = spawnSync(process.env.GLYPHSHIFT_MV_REMOTE_HELPER, [
              process.env.GLYPHSHIFT_MV_PID, process.env.GLYPHSHIFT_MV_NATIVE_SESSION_DLL, targetPath, '0',
            ], { encoding: 'utf8', windowsHide: true, timeout: 5000 });
            expect(rejected.status).toBe(1);
            expect(rejected.stderr).toContain(reason);
            expect(rejected.stdout).toBe('');
          }
        }
        for (let cycle = 0; cycle < 4; cycle++) {
          expect((await client.runtime('session-stop')).status).toBe(0);
          await expect(client.evaluate('1+1')).rejects.toThrow('Native evaluation failed: 4');
          expect((await client.runtime('session-start')).status).toBe(0);
          expect(await client.evaluate('1+1')).toBe(2);
        }
        // Temporarily leave the fixture realm gate; its own event loop restores
        // the title. An expired queued request must never execute afterwards.
        await client.evaluate("window.gsMvLateRequest=false;document.title='Other synthetic context';setTimeout(function(){document.title='Glyphshift MV Contract';},6500);true");
        await expect(client.evaluate('gsMvLateRequest=true;true')).rejects.toThrow('Native evaluation failed: 7');
        expect(await client.evaluate('gsMvLateRequest')).toBe(false);
        fs.writeFileSync(testInfo.outputPath('native-session-lifecycle.json'), JSON.stringify({
          nativeHookCycles: 4, stoppedRequests: 'rejected', wrongRealmTimeout: 'passed',
          expiredRequestExecuted: false, driverOwnsGameHooks: false,
          driverUsesFrida: !process.env.GLYPHSHIFT_MV_REMOTE_HELPER,
        }, null, 2));
      }
    }
  } finally {
    await client.evaluate('if(window.gsMvSession)gsMvSession.stop();true').catch(() => {});
    if (runtimeRoot) await client.runtime('deactivate').catch(() => {});
    await client.close();
  }
});
