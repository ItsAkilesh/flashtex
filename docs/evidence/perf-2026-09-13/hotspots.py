"""Inclusive/self sample table from inferno folded stacks.

usage: hotspots.py folded.txt [top]
Frames are `binary`symbol`; Rust v0 symbols are shortened to their path.
"""
import re
import sys
from collections import Counter


def short(frame):
    frame = frame.split("`", 1)[-1]
    # Rust v0 mangling: pull out the identifier chunks (<len><ident>).
    if frame.startswith("_R"):
        parts = []
        i = 0
        s = frame
        while i < len(s):
            m = re.match(r"(\d+)", s[i:])
            if m:
                n = int(m.group(1))
                start = i + len(m.group(1))
                ident = s[start:start + n]
                if n > 0 and re.fullmatch(r"_?[A-Za-z][A-Za-z0-9_]*", ident or "") and not re.fullmatch(r"Cs[0-9A-Za-z_]+", ident):
                    parts.append(ident)
                i = start + max(n, 0) if n > 0 else start
            else:
                i += 1
        keep = [p for p in parts if p not in ("core", "alloc", "std")]
        return "::".join(keep[-3:]) if keep else frame[:60]
    return frame[:60]


def main():
    path = sys.argv[1]
    top = int(sys.argv[2]) if len(sys.argv) > 2 else 20
    incl = Counter()
    selfc = Counter()
    total = 0
    for line in open(path):
        line = line.rstrip("\n")
        if not line:
            continue
        stack, _, count = line.rpartition(" ")
        count = int(count)
        total += count
        frames = [short(f) for f in stack.split(";")]
        for f in set(frames):
            incl[f] += count
        selfc[frames[-1]] += count
    print(f"total samples: {total}")
    print("| inclusive % | self % | frame |")
    print("| ---: | ---: | --- |")
    skip = ("main", "start", "lang_start", "__rust_begin_short_backtrace", "call_once", "thread_start", "_pthread_start")
    rows = [(c, f) for f, c in incl.items() if not any(f.endswith(s) for s in skip)]
    rows.sort(reverse=True)
    for c, f in rows[:top]:
        print(f"| {100 * c / total:.1f} | {100 * selfc[f] / total:.1f} | `{f}` |")


main()
