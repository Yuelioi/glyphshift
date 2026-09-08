'use strict';
let listeners = [];
let phase = 'baseline';
let config;
let seen = new Map();
let names = new Map();
let rejected = 0;
let replacement = null;
const activeQueries = new Map();

function boundedText(address, wide, limit) {
    if (address.isNull()) return null;
    const units = [];
    for (let i = 0; i < limit; i++) {
        const value = wide ? address.add(i * 2).readU16() : address.add(i).readU8();
        if (value === 0) return wide ? String.fromCharCode(...units) : address.readUtf8String(i);
        units.push(value);
    }
    throw new Error('unterminated or oversized text');
}
function record(kind, token, index, result, callPhase) {
    try {
        const text = boundedText(result, true, 2048);
        const event = { phase: callPhase, kind, token, index, text };
        const key = JSON.stringify(event);
        if (seen.has(key)) { seen.set(key, seen.get(key) + 1); return; }
        if (seen.size >= 512) { rejected++; return; }
        seen.set(key, 1);
        send({ type: 'query', ...event });
    } catch (_) { rejected++; }
}
rpc.exports = {
    start(input) {
        if (listeners.length) throw new Error('already observing');
        if (Process.arch !== 'ia32') throw new Error('only verified x86 argument layout is enabled');
        config = input;
        const object = ptr(config.object);
        const table = object.readPointer();
        const targets = {};
        for (const kind of ['find', 'index', 'value']) {
            const entry = config.methods[kind];
            if (!entry || !Number.isInteger(entry.slot) || entry.slot < 0 || entry.slot > 255
                || !/^(?:[0-9a-f]{2}){16,64}$/i.test(entry.prefix)) {
                throw new Error('invalid method contract');
            }
            const target = table.add(entry.slot * 4).readPointer();
            if (!target.equals(ptr(entry.address))) throw new Error('interface changed');
            const owner = Process.findModuleByAddress(target);
            const range = Process.findRangeByAddress(target);
            if (!owner || owner.name.toLowerCase() !== config.module.toLowerCase()
                || !range || !range.protection.includes('x')) throw new Error('invalid method owner');
            const bytes = new Uint8Array(target.readByteArray(entry.prefix.length / 2));
            const hex = Array.from(bytes, b => b.toString(16).padStart(2, '0')).join('');
            if (hex !== entry.prefix) throw new Error('method bytes changed');
            targets[kind] = target;
        }
        if (new Set(Object.values(targets).map(p => p.toString())).size !== 3) {
            throw new Error('query methods must have distinct verified entries');
        }
        try {
            for (const kind of ['find', 'index', 'value']) {
                listeners.push(Interceptor.attach(targets[kind], {
                    onEnter() {
                        this.valid = false;
                        try {
                            if (!this.context.ecx.equals(object)) return;
                            this.callPhase = phase;
                            const argument = this.context.esp.add(4).readPointer();
                            if (kind === 'value') this.index = argument.toUInt32();
                            else this.token = boundedText(argument, false, 512);
                            if (kind !== 'index') {
                                const depth = activeQueries.get(this.threadId) ?? 0;
                                this.queryRoot = depth === 0;
                                activeQueries.set(this.threadId, depth + 1);
                            }
                            this.valid = true;
                        } catch (_) { rejected++; }
                    },
                    onLeave(result) {
                        if (!this.valid) return;
                        if (kind === 'index') {
                            const index = result.toUInt32();
                            if (index !== 0xffffffff && names.size < 2048) names.set(index, this.token);
                            return;
                        }
                        const token = kind === 'find' ? this.token : (names.get(this.index) ?? null);
                        try {
                            record(kind, token, kind === 'value' ? this.index : null, result, this.callPhase);
                            if (replacement && this.queryRoot) replacement.apply(token, result);
                        } finally {
                            const depth = (activeQueries.get(this.threadId) ?? 1) - 1;
                            if (depth > 0) activeQueries.set(this.threadId, depth);
                            else activeQueries.delete(this.threadId);
                        }
                    }
                }));
            }
        } catch (error) {
            for (const listener of listeners) listener.detach();
            listeners = [];
            throw error;
        }
        return { observing: true, methods: listeners.length };
    },
    phase(value) { phase = String(value).slice(0, 64); },
    stop() {
        if (replacement) replacement.disable();
        for (const listener of listeners) listener.detach();
        listeners = [];
        const result = { unique: seen.size, calls: [...seen.values()].reduce((a, b) => a + b, 0), rejected };
        seen.clear(); names.clear(); activeQueries.clear();
        return result;
    }
};
