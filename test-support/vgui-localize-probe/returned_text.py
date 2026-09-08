"""Controller-owned allocations whose lifetime is the target process lifetime."""
import ctypes as C
import mmap


class ReturnedText:
    def __init__(self, pid):
        self.kernel = C.WinDLL('kernel32', use_last_error=True)
        for name, arguments, result in (
            ('OpenProcess', [C.c_uint32, C.c_int, C.c_uint32], C.c_void_p),
            ('VirtualAllocEx', [C.c_void_p, C.c_void_p, C.c_size_t, C.c_uint32, C.c_uint32], C.c_void_p),
            ('WriteProcessMemory', [C.c_void_p, C.c_void_p, C.c_void_p, C.c_size_t, C.c_void_p], C.c_int),
            ('CloseHandle', [C.c_void_p], C.c_int),
        ):
            function = getattr(self.kernel, name)
            function.argtypes, function.restype = arguments, result
        self.handle = self.kernel.OpenProcess(0x438, False, pid)
        if not self.handle:
            raise C.WinError(C.get_last_error())
        self.pages = 0
        self.generation = 0

    def publish(self, script, policy):
        generation = policy.get('generation')
        if type(generation) is not int or not self.generation < generation <= 2**53 - 1:
            raise ValueError('publication generation must increase')
        for key in ('token', 'source', 'translation'):
            value = policy.get(key)
            if not isinstance(value, str) or not value or len(value.encode('utf-16-le')) > 512:
                raise ValueError('invalid single-label text')
            if any(ord(unit) < 32 or ord(unit) == 127 for unit in value):
                raise ValueError('control characters are not supported')
        if not policy['token'].lstrip('#') or '%' in policy['source'] or '%' in policy['translation']:
            raise ValueError('empty token or formatted text is not supported')
        if self.pages >= 16:
            raise ValueError('publication allocation budget exhausted')
        data = policy['translation'].encode('utf-16-le') + b'\0\0'
        pointer = self.kernel.VirtualAllocEx(self.handle, None, mmap.PAGESIZE, 0x3000, 0x04)
        if not pointer:
            raise C.WinError(C.get_last_error())
        self.pages += 1
        # Even an ambiguous RPC failure must not free a possibly published pointer.
        # At most sixteen pages are committed during this bounded session.
        count = C.c_size_t()
        if not self.kernel.WriteProcessMemory(self.handle, pointer, data, len(data), C.byref(count)):
            raise C.WinError(C.get_last_error())
        if count.value != len(data):
            raise RuntimeError('incomplete returned-text write')
        result = script.exports_sync.publish({**policy, 'pointer': hex(pointer)})
        self.generation = generation
        return result

    def close(self):
        if self.handle:
            self.kernel.CloseHandle(self.handle)
            self.handle = None
