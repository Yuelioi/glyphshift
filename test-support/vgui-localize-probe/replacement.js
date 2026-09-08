// Explicitly opted-in, single-label experiment. The original table is untouched.
// Native pages deliberately outlive this script: callers may retain old pointers.
(() => {
    let policy = null;
    let generation = 0;
    let pages = 0;
    let changedBuffers = 0;
    const hits = {};
    function label(value) {
        return typeof value === 'string' && value.length > 0 && value.length <= 256
            && !/[\u0000-\u001f\u007f]/.test(value);
    }
    const status = () => ({ generation, pages, enabled: policy !== null, hits: { ...hits }, changedBuffers });
    replacement = {
        disable() { policy = null; return status(); },
        apply(token, result) {
            const selected = policy;
            if (!selected || token === null || token.replace(/^#/, '') !== selected.token) return;
            try {
                if (boundedText(result, true, 2048) !== selected.source) return;
                if (boundedText(selected.pointer, true, 257) !== selected.translation) {
                    changedBuffers++;
                    policy = null;
                    send({ type: 'replacement-disabled', reason: 'caller-modified-buffer' });
                    return;
                }
                result.replace(selected.pointer);
                hits[selected.generation] = (hits[selected.generation] ?? 0) + 1;
            } catch (_) {
                policy = null;
                send({ type: 'replacement-disabled', reason: 'unreadable-text' });
            }
        }
    };
    Object.assign(rpc.exports, {
        publish(input) {
            if (!config?.allowReplacement || listeners.length === 0) throw new Error('replacement not enabled');
            if (!Number.isSafeInteger(input.generation) || input.generation <= generation
                || !label(input.token) || !label(input.source) || !label(input.translation)
                || input.source.includes('%') || input.translation.includes('%')) {
                throw new Error('invalid single-label publication');
            }
            const token = input.token.replace(/^#/, '');
            if (!token || pages >= 16) throw new Error('publication budget exhausted or empty token');
            const pointer = ptr(input.pointer);
            const range = Process.findRangeByAddress(pointer);
            if (!range || range.protection !== 'rw-' || !range.base.equals(pointer)
                || range.size < (input.translation.length + 1) * 2
                || boundedText(pointer, true, 257) !== input.translation) {
                throw new Error('invalid controller-owned returned-text page');
            }
            pages++;
            generation = input.generation;
            policy = { token, source: input.source, translation: input.translation, generation, pointer };
            return status();
        },
        disable() { return replacement.disable(); },
        status
    });
})();
