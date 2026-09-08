"""Bounded diagnostic observer; all machine inputs and outputs are explicit."""
import argparse
import ctypes as C
import json
import os
from pathlib import Path
import subprocess
import sys
import threading
import time

if os.environ.get('GLYPHSHIFT_FRIDA_PYTHONPATH'):
    sys.path.insert(0, os.environ['GLYPHSHIFT_FRIDA_PYTHONPATH'])
import frida
from returned_text import ReturnedText


def read_memory(pid, address, size):
    kernel = C.WinDLL('kernel32', use_last_error=True)
    kernel.OpenProcess.argtypes = [C.c_uint32, C.c_int, C.c_uint32]
    kernel.OpenProcess.restype = C.c_void_p
    kernel.ReadProcessMemory.argtypes = [C.c_void_p, C.c_void_p, C.c_void_p, C.c_size_t, C.c_void_p]
    kernel.CloseHandle.argtypes = [C.c_void_p]
    handle = kernel.OpenProcess(0x410, False, pid)
    if not handle:
        raise C.WinError(C.get_last_error())
    try:
        buffer = C.create_string_buffer(size)
        if not kernel.ReadProcessMemory(handle, address, buffer, size, None):
            raise C.WinError(C.get_last_error())
        return buffer.raw
    finally:
        kernel.CloseHandle(handle)


def main():
    parser = argparse.ArgumentParser()
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument('--fixture', type=Path)
    group.add_argument('--config', type=Path)
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--seconds', type=float, default=30)
    parser.add_argument('--phase-file', type=Path)
    parser.add_argument('--allow-replacement', action='store_true')
    parser.add_argument('--control-file', type=Path)
    args = parser.parse_args()
    if not 1 <= args.seconds <= 120:
        parser.error('duration must be between 1 and 120 seconds')
    local_root = Path(__file__).resolve().parents[2] / 'local-test'
    for path in (args.config, args.phase_file, args.control_file):
        if path and not path.resolve().is_relative_to(local_root.resolve()):
            parser.error('machine configuration must stay under local-test')
    if args.control_file and not args.allow_replacement:
        parser.error('control commands require explicit --allow-replacement')
    output = args.output.resolve()
    if not output.is_relative_to(local_root.resolve()):
        parser.error('raw evidence must stay under local-test')
    output.parent.mkdir(parents=True, exist_ok=True)
    child = session = script = returned_text = None
    records = []
    errors = []
    lock = threading.Lock()
    try:
        if args.fixture:
            child = subprocess.Popen([str(args.fixture.resolve())], stdin=subprocess.PIPE,
                                     stdout=subprocess.PIPE, text=True, encoding='utf-8')
            object_address = int(child.stdout.readline().strip())
            pid = child.pid
            table = int.from_bytes(read_memory(pid, object_address, 4), 'little')
            methods = {}
            for slot, kind in enumerate(('find', 'index', 'value')):
                address = int.from_bytes(read_memory(pid, table + slot * 4, 4), 'little')
                methods[kind] = dict(slot=slot, address=hex(address), prefix=read_memory(pid, address, 16).hex())
            config = dict(pid=pid, object=hex(object_address), module=args.fixture.name, methods=methods)
        else:
            config = json.loads(args.config.read_text(encoding='utf-8-sig'))
        config['allowReplacement'] = args.allow_replacement
        if args.allow_replacement:
            returned_text = ReturnedText(config['pid'])
        with output.open('w', encoding='utf-8') as log:
            def message(payload, data):
                with lock:
                    if log.closed:
                        return
                    log.write(json.dumps(payload, ensure_ascii=False) + '\n')
                    log.flush()
                    if payload.get('type') == 'send':
                        records.append(payload['payload'])
                    elif payload.get('type') == 'error':
                        errors.append(payload)
            session = frida.attach(config['pid'])
            source = Path(__file__).with_name('probe.js').read_text(encoding='utf-8')
            if args.allow_replacement:
                source += '\n' + Path(__file__).with_name('replacement.js').read_text(encoding='utf-8')
            script = session.create_script(source)
            script.on('message', message)
            script.load()
            if child:
                bad = json.loads(json.dumps(config))
                prefix = bad['methods']['value']['prefix']
                bad['methods']['value']['prefix'] = ('00' if prefix[:2] != '00' else 'ff') + prefix[2:]
                try:
                    script.exports_sync.start(bad)
                except frida.core.RPCException:
                    pass
                else:
                    raise AssertionError('a changed method must be rejected')
                assert script.exports_sync.stop()['calls'] == 0
                print('PASS changed method rejected before attachment', flush=True)
            print(json.dumps(script.exports_sync.start(config)), flush=True)
            if child:
                for command in ('name', 'index'):
                    script.exports_sync.phase(command)
                    child.stdin.write(command + '\n'); child.stdin.flush()
                    assert child.stdout.readline().strip() == 'original', 'probe changed the result'
                if args.allow_replacement:
                    def ask(command):
                        child.stdin.write(command + '\n'); child.stdin.flush()
                        return child.stdout.readline().strip()
                    def publish(generation, translation):
                        return returned_text.publish(script, dict(generation=generation,
                            token='menu.open', source='Open', translation=translation))
                    publish(1, '\u7b2c\u4e00\u4ee3')
                    assert ask('copy-name') == '\u7b2c\u4e00\u4ee3'
                    assert ask('hold') == '\u7b2c\u4e00\u4ee3'
                    assert ask('table') == 'Open'
                    publish(2, '\u7b2c\u4e8c\u4ee3')
                    assert ask('cached') == '\u7b2c\u4e00\u4ee3'
                    assert ask('copy-index') == '\u7b2c\u4e8c\u4ee3'
                    assert ask('held') == '\u7b2c\u4e00\u4ee3'
                    script.exports_sync.disable()
                    assert ask('cached') == '\u7b2c\u4e8c\u4ee3'
                    assert ask('copy-index') == 'Open'
                    assert ask('held') == '\u7b2c\u4e00\u4ee3'
                    publish(3, '\u7b2c\u4e09\u4ee3')
                    assert ask('hold') == '\u7b2c\u4e09\u4ee3'
                    assert ask('mutate-held') == 'X\u4e09\u4ee3'
                    assert ask('copy-index') == 'Open'
                    assert script.exports_sync.status()['changedBuffers'] == 1
                    for generation in range(4, 17):
                        publish(generation, 'Bounded')
                    for generation in (16, 17):
                        try: publish(generation, 'Rejected')
                        except (ValueError, frida.core.RPCException): pass
                        else: raise AssertionError('stale or over-budget publication accepted')
                    script.exports_sync.disable()
                    assert ask('table') == 'Open'
                    print('PASS replacement, update, cached-control restoration, pointer lifetime and budget', flush=True)
                summary = script.exports_sync.stop()
                # Round-trip after detach: original calls still run unmodified.
                child.stdin.write('name\n'); child.stdin.flush()
                assert child.stdout.readline().strip() == 'original'
                assert script.exports_sync.stop()['calls'] == 0, 'observer remained attached'
                assert any(r.get('kind') == 'find' and r.get('text') == 'Open' for r in records)
                assert any(r.get('kind') == 'value' and r.get('token') == 'menu.open'
                           and r.get('text') == 'Open' for r in records)
                assert not errors, errors
                print('PASS native name/index observation and unchanged results before/after detach', flush=True)
                if args.allow_replacement:
                    script.unload(); script = None
                    assert ask('held') == 'X\u4e09\u4ee3', 'returned memory did not survive script unload'
                    assert ask('copy-index') == 'Open'
                    print('PASS native returned memory survives instrumentation unload', flush=True)
            else:
                deadline = time.monotonic() + args.seconds
                previous = 'baseline'
                previous_control = None
                while time.monotonic() < deadline:
                    if args.phase_file and args.phase_file.exists():
                        value = args.phase_file.read_text(encoding='utf-8-sig').strip()
                        if value != previous:
                            script.exports_sync.phase(value); previous = value
                    if args.control_file and args.control_file.exists():
                        try:
                            command = json.loads(args.control_file.read_text(encoding='utf-8-sig'))
                        except json.JSONDecodeError:
                            time.sleep(0.1)
                            continue
                        if command['id'] != previous_control:
                            action = command['action']
                            if action == 'publish': state = returned_text.publish(script, command['policy'])
                            elif action == 'disable': state = script.exports_sync.disable()
                            elif action == 'status': state = script.exports_sync.status()
                            elif action == 'stop': state = script.exports_sync.disable()
                            else: raise ValueError('unknown control action')
                            previous_control = command['id']
                            ack = args.control_file.with_suffix('.ack.json')
                            temporary = ack.with_suffix('.tmp')
                            temporary.write_text(json.dumps(dict(id=previous_control, state=state)), encoding='utf-8')
                            temporary.replace(ack)
                            if action == 'stop': break
                    time.sleep(0.25)
                summary = script.exports_sync.stop()
            print(json.dumps(summary), flush=True)
            if errors:
                raise RuntimeError('observer reported script errors; inspect local evidence')
    finally:
        if returned_text:
            returned_text.close()
        if script:
            try: script.unload()
            except (frida.InvalidOperationError, frida.TransportError): pass
        try:
            if child and child.poll() is None:
                # Do not race agent detachment with ExitProcess in the fixture.
                # Returned-pointer checks above already run after script unload.
                child.stdin.write('quit\n'); child.stdin.flush()
                try:
                    child.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    child.kill()  # Only the synthetic process created by this harness.
                    child.wait(timeout=5)
                    print('Fixture exit output:', child.stdout.read(), flush=True)
                    raise RuntimeError('owned fixture failed to exit normally')
        finally:
            if session:
                try: session.detach()
                except (frida.InvalidOperationError, frida.TransportError): pass


if __name__ == '__main__':
    main()
