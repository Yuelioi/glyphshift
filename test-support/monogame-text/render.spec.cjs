const { test, expect } = require('../../apps/glyphshift-desktop/node_modules/@playwright/test');
const { spawn, execFile } = require('node:child_process');
const { createInterface } = require('node:readline');
const { once } = require('node:events');
const { promisify } = require('node:util');
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const root = path.resolve(__dirname, '../..');
const evidence = path.join(root, 'local-test/evidence/monogame-text');
const managed = path.join(root, 'local-test/monogame-text/managed/bin');
const production = process.env.GLYPHSHIFT_MONOGAME_PRODUCTION === '1';
const actualRuntime = Boolean(process.env.GLYPHSHIFT_TEST_RUNTIME);
const native = production ? path.join(root, 'local-test/monogame-native/glyphshift_adapter_monogame_native.dll')
  : path.join(root, 'local-test/coreclr-late-attach/glyphshift_coreclr_fixture.dll');
const hostAssembly = path.join(managed, 'Host/release/Glyphshift.MonoGame.SyntheticHost.dll');
const framework = path.join(managed, 'Host/release/MonoGame.Framework.dll');
const dotnet = process.env.GLYPHSHIFT_TEST_DOTNET;
const fileHash = file => crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');

test('MonoGame pixels, metrics, updates and recovery through native Adapter ABI', async () => {
  expect(dotnet, 'test runner supplies its resolved SDK executable').toBeTruthy();
  fs.mkdirSync(evidence, { recursive: true });
  const originals = [fileHash(hostAssembly), fileHash(framework)];
  const childEnv = { ...process.env };
  for (const key of Object.keys(childEnv)) if (/^(CORECLR_|COR_|COMPlus_|DOTNET_)/i.test(key)) delete childEnv[key];
  const child = spawn(dotnet, [hostAssembly, evidence], { windowsHide: true, env: childEnv, stdio: ['pipe', 'pipe', 'pipe'] });
  let stderr = '';
  child.stderr.on('data', data => { stderr += data; });
  const lines = [];
  const readers = [];
  createInterface({ input: child.stdout }).on('line', line => {
    const pending = readers.shift();
    if (pending) pending(line); else lines.push(line);
  });
  async function read() {
    if (lines.length) return lines.shift();
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => reject(new Error('Host response timeout: ' + stderr)), 15000);
      readers.push(line => { clearTimeout(timer); resolve(line); });
    });
  }
  async function rpc(command) {
    child.stdin.write(JSON.stringify(command) + '\n');
    const response = JSON.parse(await read());
    if (response.error) throw new Error(response.error);
    return response;
  }
  const records = [];
  async function render(text, mode = 'string', options = {}) {
    const response = await rpc({ command: 'render', text, mode, ...options });
    records.push({ text, mode, ...options, ...response });
    expect(response.pixels).toBeGreaterThan(0);
    expect(response.builderUnchanged).toBe(true);
    expect(response.originalFontUnchanged).toBe(true);
    expect(response.backbufferHash, 'rendered backbuffer matches captured surface').toBe(response.hash);
    return response;
  }
  const modes = ['string', 'builder', 'vector', 'builder-vector', 'scalar', 'builder-scalar'];
  function sameImage(actual, expected) {
    expect(actual.hash, 'complete GPU pixel buffer').toBe(expected.hash);
    expect([actual.width, actual.height], 'MeasureString agrees with reference').toEqual([expected.width, expected.height]);
  }
  try {
    expect(await read()).toBe('READY');
    const refs = {};
    for (const mode of modes) {
      refs[mode] = {};
      for (const text of ['AAA', '中文', '中', 'B']) refs[mode][text] = await render(text, mode);
      expect(refs[mode].AAA.hash).not.toBe(refs[mode]['中文'].hash);
      expect(refs[mode].AAA.width).not.toBe(refs[mode]['中文'].width);
    }
    const clippedOriginal = await render('AAA', 'vector', { clip: true });
    const clippedFirst = await render('中文', 'vector', { clip: true });
    const clippedSecond = await render('中', 'vector', { clip: true });
    const spacedReference = await render(' 中文 ', 'vector');
    const wrapNarrow = await render('中文\n中文', 'vector');
    const wrapWide = await render('中文中\n文', 'vector');
    const hardBreak = await render('A B\nA B', 'vector');
    const conflictSource = await render('B B A', 'vector');
    const fallbackRefs = {};
    if (production) for (const mode of modes) {
      fallbackRefs[mode] = {};
      for (const text of ['中文', '中', '斧头', 'A中B']) fallbackRefs[mode][text] = await render(text, mode, { missing: true, fallbackReference: true });
    }
    expect((await rpc({ command: 'load', path: native })).status).toBe(0);
    if (!production) await promisify(execFile)(dotnet, [path.join(managed, 'Attach/release/Attach.dll'), String(child.pid), native, path.basename(hostAssembly)], { windowsHide: true, timeout: 20000 });
    await expect.poll(async () => {
      for (const mode of modes) await render('AAA', mode);
      const result = await rpc({ command: 'status' });
      if (result.status < 0) throw new Error('Profiler failure: ' + (result.status >>> 0).toString(16));
      return result.status;
    }, { timeout: 15000 }).toBe(0);
    for (const mode of modes) {
      const before = (await rpc({ command: 'status' })).calls;
      const translated = await render('AAA', mode);
      sameImage(translated, refs[mode]['中文']);
      if (!actualRuntime) expect(translated.calls - before, 'one measurement and two leaf draws, with no wrapper double translation').toBe(3);
      sameImage(await render('B', mode), refs[mode].B);
      await render('AAA', mode, { missing: true }); // New glyphs publish at the frame boundary.
      const missingFont = await render('AAA', mode, { missing: true });
      if (production) {
        expect(missingFont.hash, 'dictionary hit renders fallback glyphs').not.toBe(refs[mode].AAA.hash);
        sameImage(missingFont, fallbackRefs[mode]['中文']);
        await render('AAA', mode, {missing: true, compressed: true});
        sameImage(await render('AAA', mode, {missing: true, compressed: true}), fallbackRefs[mode]['中文']);
      } else sameImage(missingFont, refs[mode].AAA);
    }
    sameImage(await render('AAA', 'vector', { clip: true }), clippedFirst);
    if (production) sameImage(await render(' AAA ', 'vector'), spacedReference);
    if (actualRuntime) {
      sameImage(await render('A B \r\nA B', 'vector'), wrapNarrow);
      sameImage(await render('A \r\nB A B', 'vector'), wrapWide);
      sameImage(await render('B A \r\nB', 'vector'), refs.vector['中']); // Unique legacy translation survives a new wrapping.
      sameImage(await render('B B A', 'vector'), conflictSource); // Conflicting legacy translations are not picked arbitrarily.
      sameImage(await render('B \r\nB A', 'vector'), refs.vector['中']); // Its original exact mapping still works.
      sameImage(await render('A B\nA B', 'vector'), hardBreak);
      const captured = await rpc({command:'runtime_observations'});
      const rows = JSON.parse(captured.observations).records;
      expect(rows.some(row => row.source === 'A B A B')).toBe(true);
      expect(rows.some(row => row.source === 'A B\nA B')).toBe(true);
      expect(rows.some(row => row.source === 'A B \r\nA B' || row.source === 'A \r\nB A B')).toBe(false);
    }
    await rpc({ command: 'generation', generation: 2 });
    for (const mode of modes) sameImage(await render('AAA', mode), refs[mode]['中']);
    sameImage(await render('AAA', 'vector', { clip: true }), clippedSecond);
    if (production) for (const mode of modes) sameImage(await render('AAA', mode, {missing: true}), fallbackRefs[mode]['中']);
    if (actualRuntime) {
      await rpc({command: 'generation', generation: 3});
      await render('AAA', 'vector', {missing: true}); // New glyph set becomes visible on the next frame.
      for (const mode of modes) sameImage(await render('AAA', mode, {missing: true}), fallbackRefs[mode]['斧头']);
      await rpc({command: 'generation', generation: 5});
      for (const mode of modes) sameImage(await render('AAA', mode, {missing: true, compressed: true}), fallbackRefs[mode]['A中B']);
      await rpc({command: 'generation', generation: 6});
    }
    await rpc({ command: 'gc' });
    sameImage(await render('AAA', 'builder-vector'), refs['builder-vector']['中']);
    if (actualRuntime) {
      await rpc({command:'generation',generation:7});
      sameImage(await render(' AAA ', 'vector'), refs.vector.B); // An explicit padded key wins over trimmed lookup.
    }
    if (actualRuntime) {
      const captured = await rpc({ command: 'runtime_observations' });
      expect(captured.observations).toContain('AAA');
      expect(captured.ack).toContain('windows.monogame.sprite-batch-draw-string');
    } else {
      expect((await rpc({ command: 'observe' })).status).toBe(0);
      sameImage(await render('AAA'), refs.string.AAA);
    }
    expect((await rpc({ command: 'deactivate' })).status).toBe(0);
    const stopped = (await rpc({ command: 'status' })).calls;
    for (const mode of modes) sameImage(await render('AAA', mode), refs[mode].AAA);
    for (const mode of modes) sameImage(await render('AAA', mode, {missing: true}), refs[mode].AAA);
    for (const mode of modes) sameImage(await render('AAA', mode, {missing: true, compressed: true}), refs[mode].AAA);
    sameImage(await render('AAA', 'vector', { clip: true }), clippedOriginal);
    expect((await rpc({ command: 'status' })).calls).toBe(stopped);
    expect((await rpc({ command: 'activate' })).status).toBe(0);
    sameImage(await render('AAA'), refs.string['中']);
    if (!actualRuntime) {
      expect((await rpc({ command: 'revert' })).status).toBe(0);
      expect((await rpc({ command: 'activate' })).status).toBe(production ? 3 : 0);
    } else expect((await rpc({ command: 'deactivate' })).status).toBe(0);
    for (const mode of modes) sameImage(await render('AAA', mode), refs[mode].AAA);
    expect([fileHash(hostAssembly), fileHash(framework)]).toEqual(originals);
    await rpc({ command: 'exit' });
    if (child.exitCode === null) await once(child, 'exit');
    expect(child.exitCode, stderr).toBe(0);
  } finally {
    fs.writeFileSync(path.join(evidence, 'frames.json'), JSON.stringify(records, null, 2));
    fs.writeFileSync(path.join(evidence, 'stderr.log'), stderr);
    if (child.exitCode === null) { child.kill(); await once(child, 'exit'); }
  }
});
