#!/usr/bin/env python3
"""Linux-only transparent original-producer capture for local acceptance replay."""
import ctypes
import os
from pathlib import Path
import signal
import subprocess
import sys
import threading
import time


def main():
    root = Path(os.environ['FLASHTEX_CAPTURE_DIRECTORY'])
    prefix = root / ('producer-' + str(os.getpid()))
    proxy_pid = os.getpid()
    def parent_death():
        # This experimental Linux capture must not orphan a producer on proxy kill.
        if ctypes.CDLL(None).prctl(1, signal.SIGKILL, 0, 0, 0) != 0:
            os._exit(125)
        if os.getppid() != proxy_pid:
            os._exit(125)
    child = subprocess.Popen([os.environ['FLASHTEX_CAPTURE_PRODUCER']],
        stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=sys.stderr,
        preexec_fn=parent_death)
    (prefix.with_suffix('.pid')).write_text(str(child.pid))
    def output():
        with prefix.with_suffix('.output.jsonl').open('wb') as log:
            for frame in child.stdout:
                log.write(frame); log.flush()
                if (root / 'hold-output').exists():
                    (root / 'output-held').write_text(str(os.getpid()))
                    deadline = time.monotonic() + 10
                    while (root / 'hold-output').exists():
                        if time.monotonic() > deadline:
                            os._exit(123)
                        time.sleep(.002)
                sys.stdout.buffer.write(frame); sys.stdout.buffer.flush()
    reader = threading.Thread(target=output)
    reader.start()
    try:
        with prefix.with_suffix('.input.jsonl').open('wb') as log:
            for frame in sys.stdin.buffer:
                log.write(frame); log.flush()
                child.stdin.write(frame); child.stdin.flush()
        child.stdin.close()
        child.wait(timeout=10)
    finally:
        if child.poll() is None:
            child.kill(); child.wait(timeout=5)
        reader.join(timeout=5)
        if reader.is_alive():
            os._exit(124)

if __name__ == '__main__':
    main()
