'use strict';
const { test } = require('node:test');
const assert = require('node:assert/strict');
const { installMessageAdapter } = require('./adapter.cjs');

// Synthetic message owner, not a copied engine or proof of target pixels.
function fixture(source = 'Today, we explore.\nChoose a path.') {
  const message = { source };
  class MessageWindow {
    convertEscapeCharacters(text) { return text; }
    startMessage() {
      this.starts = (this.starts ?? 0) + 1;
      this._textState = { index: 0, text: this.convertEscapeCharacters(message.source) };
      return 'started';
    }
  }
  const observations = [];
  const dictionary = new Map();
  const host = { resolve: event => {
    observations.push(event);
    return event.usage === 'draw' ? dictionary.get(event.source) : undefined;
  } };
  const window = new MessageWindow();
  return { message, MessageWindow, window, observations, dictionary, host };
}

test('captures the full multiline source before typewriter and never changes message data', () => {
  const f = fixture();
  f.dictionary.set(f.message.source, '今天，一起探索。\n选择一条路。');
  const original = f.MessageWindow.prototype.startMessage;
  const session = installMessageAdapter(f.MessageWindow, f.host);
  assert.equal(f.window.startMessage(), 'started');
  assert.equal(f.window._textState.text, '今天，一起探索。\n选择一条路。');
  assert.equal(f.window._textState.index, 0);
  assert.equal(f.message.source, 'Today, we explore.\nChoose a path.');
  assert.deepEqual(f.observations, [{ source: f.message.source, origin: 'display-entry', usage: 'draw' }]);
  assert.equal(Object.hasOwn(f.window, 'convertEscapeCharacters'), false);
  session.stop();
  assert.equal(f.MessageWindow.prototype.startMessage, original);
  f.window.startMessage();
  assert.equal(f.window._textState.text, f.message.source);
  assert.equal(f.observations.length, 1);
});

test('new lookup results affect the next message without pretending to refresh current text', () => {
  const f = fixture('Hello');
  const session = installMessageAdapter(f.MessageWindow, f.host);
  f.dictionary.set('Hello', '你好');
  f.window.startMessage();
  f.dictionary.set('Hello', '您好');
  assert.equal(f.window._textState.text, '你好');
  f.window.startMessage();
  assert.equal(f.window._textState.text, '您好');
  session.stop();
  assert.equal(f.window._textState.text, '您好');
  assert.equal(session.capabilities.refreshCurrent, false);
  assert.equal(session.capabilities.restoreCurrent, false);
  assert.equal(f.window.starts, 2);
});

test('formatting and engine control codes are observed but passed through', () => {
  for (const source of ['Hello \\N[1]', '\\C[2]Hello', 'Wait\\!', 'Pause\x1b!']) {
    const f = fixture(source);
    f.host.resolve = event => { f.observations.push(event); return 'unsafe'; };
    installMessageAdapter(f.MessageWindow, f.host);
    f.window.startMessage();
    assert.equal(f.window._textState.text, source);
    assert.equal(f.observations.length, 1);
    assert.equal(f.observations[0].usage, 'observe');
  }
});

test('invalid or control-code translations never reach the engine', () => {
  for (const value of ['', undefined, Promise.resolve('later'), '\\!Injected', 'x'.repeat(16385)]) {
    const f = fixture('Hello');
    f.host.resolve = () => value;
    installMessageAdapter(f.MessageWindow, f.host);
    f.window.startMessage();
    assert.equal(f.window._textState.text, 'Hello');
  }
});

test('lookup and observation failures preserve original game behavior', () => {
  const f = fixture();
  f.host.resolve = () => { throw new Error('disconnected'); };
  installMessageAdapter(f.MessageWindow, f.host);
  f.window.startMessage();
  assert.equal(f.window._textState.text, f.message.source);
});

test('game exceptions propagate and scoped methods are restored', () => {
  const f = fixture();
  f.MessageWindow.prototype.startMessage = function () {
    this.convertEscapeCharacters('Hello');
    throw new Error('game error');
  };
  installMessageAdapter(f.MessageWindow, f.host);
  assert.throws(() => f.window.startMessage(), /game error/);
  assert.equal(Object.hasOwn(f.window, 'convertEscapeCharacters'), false);
  assert.throws(() => f.window.startMessage(), /game error/);
  assert.equal(f.observations.length, 2);
});

test('stop does not erase a hook installed later by the game', () => {
  const f = fixture();
  const session = installMessageAdapter(f.MessageWindow, f.host);
  const wrapped = f.MessageWindow.prototype.startMessage;
  function plugin() { return wrapped.call(this); }
  f.MessageWindow.prototype.startMessage = plugin;
  session.stop();
  session.stop();
  assert.equal(f.MessageWindow.prototype.startMessage, plugin);
  f.window.startMessage();
  assert.equal(f.observations.length, 0);
});

test('locked conversion methods pass through and retain their descriptor', () => {
  const f = fixture();
  const own = { value: text => text, writable: false, configurable: false };
  Object.defineProperty(f.window, 'convertEscapeCharacters', own);
  installMessageAdapter(f.MessageWindow, f.host);
  f.window.startMessage();
  assert.equal(f.observations.length, 0);
  assert.equal(f.window.convertEscapeCharacters, own.value);
});

test('unrelated conversions outside startMessage are not observed or translated', () => {
  const f = fixture('Hello');
  f.dictionary.set('Hello', '你好');
  installMessageAdapter(f.MessageWindow, f.host);
  assert.equal(f.window.convertEscapeCharacters('Hello'), 'Hello');
  assert.equal(f.observations.length, 0);
});

test('recursive entry from a runtime callback does not reenter translation', () => {
  const f = fixture('Hello');
  const second = new f.MessageWindow();
  f.host.resolve = event => { f.observations.push(event); second.startMessage(); return '你好'; };
  installMessageAdapter(f.MessageWindow, f.host);
  f.window.startMessage();
  assert.equal(second._textState.text, 'Hello');
  assert.equal(f.window._textState.text, '你好');
  assert.equal(f.observations.length, 1);
});

test('duplicate activation is refused and clean stop permits reactivation', () => {
  const f = fixture();
  const session = installMessageAdapter(f.MessageWindow, f.host);
  assert.throws(() => installMessageAdapter(f.MessageWindow, f.host), /already installed/);
  session.stop();
  const next = installMessageAdapter(f.MessageWindow, f.host);
  f.window.startMessage();
  assert.equal(f.observations.length, 1);
  next.stop();
});

test('stop during a decision never publishes a late translation', () => {
  const f = fixture('Hello');
  const session = installMessageAdapter(f.MessageWindow, f.host);
  f.host.resolve = () => { session.stop(); return '你好'; };
  f.window.startMessage();
  assert.equal(f.window._textState.text, 'Hello');
});
