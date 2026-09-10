"""Opt-in fixture driver. Native helper or legacy Frida, with explicit target identity."""
import json
import os
from pathlib import Path
import subprocess
import sys



def main():
    sys.stdin.reconfigure(encoding="utf-8-sig")
    sys.stdout.reconfigure(encoding="utf-8")
    root_pid = int(os.environ["GLYPHSHIFT_MV_PID"])
    executable = Path(os.environ["GLYPHSHIFT_MV_EXE"]).resolve(strict=True)
    # Query only candidate runtime processes; never attach by name alone.
    command = (
        "Get-CimInstance Win32_Process -Filter \"Name = 'Game.exe'\" | "
        "Select-Object ProcessId,ParentProcessId,ExecutablePath,CommandLine,"
        "@{Name='StartedAt';Expression={"
        "$reported=$_.CreationDate.ToFileTimeUtc();"
        "$exact=(Get-Process -Id $_.ProcessId -ErrorAction Stop).StartTime.ToUniversalTime().ToFileTimeUtc();"
        "if($exact -lt $reported -or $exact -ge ($reported+10)){throw 'Process changed during discovery'};"
        "$exact}} | ConvertTo-Json"
    )
    processes = json.loads(subprocess.check_output(
        ["powershell", "-NoProfile", "-Command", command],
        encoding="utf-8-sig", timeout=10,
        creationflags=subprocess.CREATE_NO_WINDOW,
    ))
    if isinstance(processes, dict):
        processes = [processes]

    def owned(process):
        return bool(process.get("ExecutablePath")) and Path(process["ExecutablePath"]).resolve() == executable

    if not any(p["ProcessId"] == root_pid and owned(p) for p in processes):
        raise RuntimeError("Authorized runtime root no longer matches")
    targets = [p for p in processes if p["ParentProcessId"] == root_pid
               and owned(p) and "--type=renderer" in (p.get("CommandLine") or "")]
    if len(targets) != 1:
        raise RuntimeError("Expected one renderer in the authorized runtime")

    native_session = os.environ.get("GLYPHSHIFT_MV_NATIVE_SESSION_DLL")
    helper = os.environ.get("GLYPHSHIFT_MV_REMOTE_HELPER")
    session = None
    if helper:
        from remote_client import RemoteClient
        script = RemoteClient(Path(helper).resolve(strict=True), targets[0]["ProcessId"],
                              Path(native_session).resolve(strict=True), executable, targets[0]["StartedAt"])
    else:
        import frida
        session = frida.attach(targets[0]["ProcessId"])
        script_name = "native-control.frida.js" if native_session else "native-entry.frida.js"
        script = session.create_script(Path(__file__).with_name(script_name).read_text(encoding="utf-8"))
        script.on("message", lambda message, data: print(message, file=sys.stderr, flush=True))
    try:
        script.load()
        if native_session and not helper:
            script.exports_sync.configure(str(Path(native_session).resolve(strict=True)))
        for line in sys.stdin:
            item = json.loads(line)
            try:
                if "operation" in item:
                    runtime = Path(os.environ["GLYPHSHIFT_MV_RUNTIME_DLL"]).resolve(strict=True)
                    if runtime.name != "glyphshift_target_runtime.dll":
                        raise ValueError("Expected the explicitly supplied Runtime library")
                    result = script.exports_sync.runtime(str(runtime), item["operation"], item.get("json", ""))
                    print(json.dumps({"id": item["id"], "value": result}), flush=True)
                    continue
                expression = item["expression"]
                if not isinstance(expression, str) or len(expression) > 131072:
                    raise ValueError("Invalid fixture expression")
                result = json.loads(script.exports_sync.evaluate(
                    "return {value:eval(" + json.dumps(expression) + ")};"
                ))
                if "error" in result:
                    raise RuntimeError(result["error"])
                response = {"id": item["id"], "value": result.get("value")}
            except Exception as error:
                response = {"id": item["id"], "error": str(error)}
            print(json.dumps(response), flush=True)
    finally:
        try:
            script.exports_sync.stop()
        finally:
            if session:
                session.detach()
            else:
                script.close()


if __name__ == "__main__":
    main()
