#!/usr/bin/env python3
"""Repair apps/companion/FlashTeXCompanion.xcodeproj/project.pbxproj as of e7ce5b9.

Corruption (see GitHub issue #3): whole object definitions were spliced inline
into list positions (`children = (...)`, `files = (...)`, `productRefGroup = ...`),
ImageValidator appears 3x in Sources, Swift build files sit in Resources, and
BonjourTransport.swift (added in e7ce5b9) is not in the project at all.

Usage: repair_pbxproj.py <path/to/project.pbxproj>
"""
import re
import sys

path = sys.argv[1]
src = open(path, encoding="utf-8").read()

# 1. Drop the inline `A5000006 /* Services */ = { ... };` body spliced into the
#    main group's children list (indented 4 tabs; the real definition is 2 tabs).
src, n1 = re.subn(
    r"(?ms)^\t\t\t\tA5000006 /\* Services \*/ = \{\n.*?^\t\t\};\n", "", src)

# 2. productRefGroup must point at the Products group, not an inline Services body.
src, n2 = re.subn(
    r"(?ms)productRefGroup = A5000006 /\* Services \*/ = \{\n.*?^\t\t\};\n\t\tA5000005 /\* Products \*/;",
    "productRefGroup = A5000005 /* Products */;", src)

# 3. Remove one-line object definitions that were spliced into list positions.
#    Real definitions live inside their own /* Begin ... section */ at 2-tab indent
#    and end with `};` too, so restrict to lines inside a `(`...`)` list.
out, in_list, seen = [], False, set()
list_key = None
for line in src.split("\n"):
    stripped = line.strip()
    if re.match(r"^(children|files|buildConfigurations|buildPhases|targets|knownRegions|buildRules|dependencies|LD_RUNPATH_SEARCH_PATHS) = \($", stripped):
        in_list, seen, list_key = True, set(), stripped.split(" ")[0]
        out.append(line); continue
    if in_list and stripped == ");":
        in_list = False
        out.append(line); continue
    if in_list:
        if re.match(r"^[0-9A-F]{8} /\*.*\*/ = \{isa = .*\};$", stripped):
            continue  # spliced inline definition -> drop
        m = re.match(r"^([0-9A-F]{8}) /\*.*\*/,$", stripped)
        if m:
            if m.group(1) in seen:
                continue  # duplicate list entry -> drop
            seen.add(m.group(1))
            line = "\t\t\t\t" + stripped  # normalise indentation
    out.append(line)
src = "\n".join(out)

# 4. Resources phase must contain only the asset catalog.
src = re.sub(
    r"(?ms)(A7000002 /\* Resources \*/ = \{.*?files = \(\n).*?(\t\t\t\);)",
    r"\1\t\t\t\tA1000009 /* Assets.xcassets in Resources */,\n\2", src)

# 5. Add BonjourTransport.swift (Services/) — new in e7ce5b9 but never added.
if "BonjourTransport.swift" not in src:
    src = src.replace(
        "\t\tA1000009 /* Assets.xcassets in Resources */ = {isa = PBXBuildFile; fileRef = A2000009; };\n",
        "\t\tA1000009 /* Assets.xcassets in Resources */ = {isa = PBXBuildFile; fileRef = A2000009; };\n"
        "\t\tA100000D /* BonjourTransport.swift in Sources */ = {isa = PBXBuildFile; fileRef = A200000E; };\n")
    src = src.replace(
        "\t\tA200000A /* Info.plist */ = {isa = PBXFileReference; lastKnownFileType = text.plist.xml; path = Info.plist; sourceTree = \"<group>\"; };\n",
        "\t\tA200000A /* Info.plist */ = {isa = PBXFileReference; lastKnownFileType = text.plist.xml; path = Info.plist; sourceTree = \"<group>\"; };\n"
        "\t\tA200000E /* BonjourTransport.swift */ = {isa = PBXFileReference; lastKnownFileType = sourcecode.swift; path = BonjourTransport.swift; sourceTree = \"<group>\"; };\n")
    src = src.replace(
        "\t\t\t\tA200000D /* ImageValidator.swift */,\n\t\t\t);\n\t\t\tpath = Services;",
        "\t\t\t\tA200000D /* ImageValidator.swift */,\n\t\t\t\tA200000E /* BonjourTransport.swift */,\n\t\t\t);\n\t\t\tpath = Services;")
    src = src.replace(
        "\t\t\t\tA100000C /* ImageValidator.swift in Sources */,\n\t\t\t);",
        "\t\t\t\tA100000C /* ImageValidator.swift in Sources */,\n\t\t\t\tA100000D /* BonjourTransport.swift in Sources */,\n\t\t\t);")

open(path, "w", encoding="utf-8").write(src)
print(f"repaired: removed {n1} spliced Services body, fixed productRefGroup x{n2}")
