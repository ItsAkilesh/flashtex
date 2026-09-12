"""Measure complete JSONL exchanges with bounded I/O and child cleanup."""
import argparse
import json
import os
from pathlib import Path
import selectors
import subprocess
import time


def exchange(process, request, timeout, max_reply):
    start = time.perf_counter()
    deadline = start + timeout
    sent = 0
    reply = bytearray()
    with selectors.DefaultSelector() as events:
        events.register(process.stdin, selectors.EVENT_WRITE)
        events.register(process.stdout, selectors.EVENT_READ)
        while True:
            remaining = deadline - time.perf_counter()
            if remaining <= 0:
                raise TimeoutError("complete request/response deadline exceeded")
            ready = events.select(remaining)
            if not ready:
                raise TimeoutError("complete request/response deadline exceeded")
            for key, _ in ready:
                if key.fileobj is process.stdin:
                    try:
                        sent += os.write(process.stdin.fileno(), request[sent:sent + 65536])
                    except BlockingIOError:
                        continue
                    if sent == len(request):
                        events.unregister(process.stdin)
                else:
                    try:
                        chunk = os.read(process.stdout.fileno(), min(65536, max_reply + 1 - len(reply)))
                    except BlockingIOError:
                        continue
                    if not chunk:
                        raise EOFError("worker closed stdout before a complete response")
                    reply.extend(chunk)
                    if len(reply) > max_reply:
                        raise ValueError("response exceeds configured byte bound")
                    if b"\n" in reply:
                        if sent != len(request) or reply.index(b"\n") != len(reply) - 1:
                            raise ValueError("unexpected response framing")
                        return bytes(reply), (time.perf_counter() - start) * 1000


def run(binary, requests, expected, output, timeout=30.0, max_reply=16 * 1024 * 1024):
    output = Path(output)
    output.mkdir(exist_ok=True)
    rows = []
    if timeout <= 0 or max_reply <= 0:
        raise ValueError("timeout and reply bound must be positive")
    with open(requests, 'rb') as inputs, open(expected, 'rb') as references, \
            open(output / 'stderr.txt', 'wb') as errors, \
            open(output / 'response.jsonl', 'wb') as captured:
        process = subprocess.Popen([str(binary)], stdin=subprocess.PIPE,
                                   stdout=subprocess.PIPE, stderr=errors, bufsize=0)
        try:
            os.set_blocking(process.stdin.fileno(), False)
            os.set_blocking(process.stdout.fileno(), False)
            for request in inputs:
                reference = references.readline()
                if not reference or not request.endswith(b'\n'):
                    raise ValueError("request/reference line mismatch")
                reply, elapsed = exchange(process, request, timeout, max_reply)
                captured.write(reply)
                if json.loads(reply) != json.loads(reference):
                    raise ValueError("complete response differs from reference")
                rows.append({'elapsed_ms': elapsed, 'request_bytes': len(request),
                             'reply_bytes': len(reply)})
            if references.read(1):
                raise ValueError("extra reference responses")
            process.stdin.close()
            if process.wait(timeout=timeout) != 0:
                raise RuntimeError("worker exited unsuccessfully")
        finally:
            if process.poll() is None:
                process.kill()
            process.wait(timeout=5)
            if not process.stdin.closed:
                process.stdin.close()
            process.stdout.close()
    (output / 'result.json').write_text(json.dumps(rows, indent=2) + '\n')
    return rows


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['binary', 'requests', 'expected', 'output']:
        parser.add_argument('--' + name, required=True)
    parser.add_argument('--timeout', type=float, default=30.0)
    parser.add_argument('--max-reply', type=int, default=16 * 1024 * 1024)
    arguments = parser.parse_args()
    run(**vars(arguments))
