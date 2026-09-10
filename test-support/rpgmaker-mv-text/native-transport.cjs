'use strict';
const { spawn } = require('node:child_process');
const readline = require('node:readline');

// Explicitly selected local driver; it owns PID/path validation and Frida attach.
// No HTTP listener or game debug port is required.
function connectNative(driver) {
  const child = spawn(process.env.GLYPHSHIFT_TEST_PYTHON || 'python', [driver], {
    windowsHide: true, stdio: ['pipe', 'pipe', 'inherit'],
    env: { ...process.env, PYTHONUTF8: '1' },
  });
  const pending = new Map();
  let sequence = 0;
  function fail(error) {
    for (const request of pending.values()) { clearTimeout(request.timer); request.reject(error); }
    pending.clear();
  }
  child.on('error', fail);
  child.on('exit', () => fail(new Error('Native test driver exited')));
  readline.createInterface({ input: child.stdout }).on('line', line => {
    let message;
    try { message = JSON.parse(line); } catch { fail(new Error('Invalid native driver response')); return; }
    const request = pending.get(message.id);
    if (!request) return;
    pending.delete(message.id); clearTimeout(request.timer);
    if (message.error) request.reject(new Error(message.error));
    else request.resolve(message.value);
  });
  function request(message) {
    return new Promise((resolve, reject) => {
      const id = ++sequence;
      const timer = setTimeout(() => { pending.delete(id); reject(new Error('Native driver timeout')); }, 12000);
      pending.set(id, { resolve, reject, timer });
      child.stdin.write(JSON.stringify({ id, ...message }) + '\n');
    });
  }
  function evaluate(expression) { return request({ expression }); }
  return {
    evaluate,
    runtime(operation, json = '') { return request({ operation, json }); },
    async capture() {
      // Engine framebuffer snapshot, not an OS-window screenshot.
      return (await evaluate('SceneManager.snap()._canvas.toDataURL("image/png")')).split(',')[1];
    },
    async close() {
      child.stdin.end();
      if (child.exitCode !== null) return;
      await new Promise(resolve => {
        const timer = setTimeout(() => { child.kill(); resolve(); }, 7000);
        child.once('exit', () => { clearTimeout(timer); resolve(); });
      });
    },
  };
}
module.exports = { connectNative };
