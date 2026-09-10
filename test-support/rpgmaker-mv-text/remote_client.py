"""Standard-library-only client for the explicit x86 fixture helper."""
import json
import os
import struct
import subprocess


class RemoteClient:
    def __init__(self, helper, pid, library, executable, started):
        self.child = subprocess.Popen(
            [str(helper), str(pid), str(library), str(executable), str(started)],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE,
            creationflags=subprocess.CREATE_NO_WINDOW,
        )
        self.exports_sync = self

    def _read(self, length):
        chunks = bytearray()
        while len(chunks) < length:
            data = self.child.stdout.read(length - len(chunks))
            if not data:
                raise RuntimeError("Native fixture helper exited")
            chunks.extend(data)
        return bytes(chunks)

    def _request(self, operation, first="", second=""):
        if self.child.poll() is not None:
            raise RuntimeError("Native fixture helper exited")
        first = first.encode("utf-8")
        second = second.encode("utf-8")
        if max(len(first), len(second)) > 4 * 1024 * 1024:
            raise ValueError("Fixture command too large")
        self.child.stdin.write(struct.pack("<III", operation, len(first), len(second)) + first + second)
        self.child.stdin.flush()
        status, length = struct.unpack("<II", self._read(8))
        if length > 4 * 1024 * 1024:
            raise RuntimeError("Invalid helper response")
        return status, self._read(length).decode("utf-8")

    def load(self):
        if os.environ.get("GLYPHSHIFT_MV_MANAGED_BOOTSTRAP"):
            return  # Runtime activation owns game-thread bootstrap in this mode.
        status, _ = self._request(1)
        if status:
            raise RuntimeError(f"Native session start failed: {status}")

    def evaluate(self, source):
        status, result = self._request(3, source)
        if status:
            raise RuntimeError(f"Native evaluation failed: {status}")
        return result

    def runtime(self, path, operation, data):
        if operation == "preload":
            return {"status": 0}  # helper retains the fixture DLL
        operations = {"session-start": 1, "session-stop": 2, "activate": 4,
                      "update": 5, "capture": 6, "deactivate": 7}
        status, result = self._request(operations[operation], path, data)
        answer = {"status": status}
        if operation == "capture":
            answer["batch"] = json.loads(result) if status == 0 else None
        return answer

    def stop(self):
        if self.child.poll() is not None:
            return
        status, _ = self._request(2)
        if status:
            raise RuntimeError(f"Native session stop failed: {status}")

    def close(self):
        try:
            self.child.stdin.close()
        except OSError:
            pass
        try:
            self.child.wait(timeout=20)
        except subprocess.TimeoutExpired:
            self.child.kill()
            self.child.wait()
