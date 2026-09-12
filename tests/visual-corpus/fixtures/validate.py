#!/usr/bin/env python3
"""Check visual fixture integrity only; never pretend this compiles LaTeX."""
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
manifest = json.loads((root / 'manifest.json').read_text())
assert manifest['schema_version'] == 1
ids = set()
for case in manifest['cases']:
    assert case['id'] not in ids
    ids.add(case['id'])
    path = Path(case['entry_path'])
    assert path.name == str(path) and path.suffix == '.tex'
    source = (root / path).read_bytes()
    assert hashlib.sha256(source).hexdigest() == case['source_sha256']
    text = source.decode('utf-8')
    assert text.count('\\begin{document}') == text.count('\\end{document}') == 1
    assert text.count('\\newpage') + 1 == case['expected_page_count']
    assert case['reference_state'] == 'not_generated'
    profile = manifest['profiles'][case['profile']]
    assert profile['dpi'] == 144 and profile['expected_page_pixels'] == [1224, 1584]
    assert profile['comparison']['required_differing_pixel_count'] == 0
    assert case['features'] and case['comparison_gates']
assert {p.name for p in root.glob('*.tex')} == {c['entry_path'] for c in manifest['cases']}
print(f'Validated {len(ids)} fixture sources and declared comparison profiles; TeX syntax/rendering NOT verified.')
